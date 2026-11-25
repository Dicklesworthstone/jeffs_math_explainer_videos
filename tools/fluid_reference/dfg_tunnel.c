/*
 * DFG cylinder channel on the pinned Basilisk quadtree.
 * Fixed and adaptive runs share one binary, geometry, pressure gauge,
 * flow discretization, diagnostic definitions and observation clock.
 */
#include "grid/quadtree.h"
#include "embed.h"
#include "navier-stokes/centered.h"
#include <errno.h>
#include <stdint.h>
#include <time.h>

static const double length = 2.2, height = 0.41, box = 2.56;
static const double xc = 0.2, yc = 0.2, diameter = 0.1, nu = 0.001;
static const int channel_level = 8;
static bool cylinder, adaptive;
static int max_level, adapt_interval;
static double umax, eta;
/* qcc classifies these event expressions before main reads the options. */
static double duration = 1., sample_interval = .025;
static face vector dynamic_viscosity[];
static scalar transfer_pressure[];
static FILE *series, *transfers;
static long long leaf_steps, allocated_steps;
static long wet, allocated, completed_steps, changes, capped_solves;
static double maximum_weighted_div, maximum_pressure_residual, maximum_cfl;
static double elapsed_start, minimum_step = HUGE_VAL, maximum_step;
static const char *data_directory;
static int capture_every, observations;
static double capture_after, last_sample = -1.;

static double inlet(double y)
{
  return 4. * umax * y * (height - y) / sq(height);
}

u.n[left] = dirichlet(inlet(y));
u.t[left] = dirichlet(0.);
p[left] = neumann(0.);
pf[left] = neumann(0.);
transfer_pressure[left] = neumann(0.);

u.n[right] = neumann(0.);
u.t[right] = neumann(0.);
p[right] = dirichlet(0.);
pf[right] = dirichlet(0.);
transfer_pressure[right] = dirichlet(0.);

u.n[top] = dirichlet(0.);
u.t[top] = dirichlet(0.);
u.n[bottom] = dirichlet(0.);
u.t[bottom] = dirichlet(0.);
u.n[embed] = dirichlet(0.);
u.t[embed] = dirichlet(0.);

static double monotonic_time(void)
{
  struct timespec now;
  if (clock_gettime(CLOCK_MONOTONIC, &now))
    abort();
  return now.tv_sec + 1.e-9 * now.tv_nsec;
}

static void refresh_mesh_counts(void)
{
  long count = 0;
  foreach (reduction(+:count))
    if (cs[] > 0.)
      count++;
  wet = count;
  /* Layer.nc counts allocated child blocks. Include internal cells, halo
     cells and masked storage that the pinned tree actually allocated. */
  allocated = 0;
  for (int l = 0; l <= grid->depth; l++)
    allocated += l == 0 ? sq(tree->L[l]->len) : 4 * tree->L[l]->nc;
}

typedef struct {
  double weighted_max, full_cell_max, wet_l2, net, energy, volume;
} Flow;

static Flow flow_diagnostics(void)
{
  double weighted_max = 0., full_max = 0., sum2 = 0., net = 0.;
  double energy = 0., volume = 0.;
  foreach (reduction(max:weighted_max) reduction(max:full_max)
           reduction(+:sum2) reduction(+:net) reduction(+:energy) reduction(+:volume)) {
    if (cs[] > 0.) {
      double div = 0.;
      foreach_dimension()
        div += uf.x[1] - uf.x[];
      div /= Delta;
      double area = sq(Delta);
      weighted_max = max(weighted_max, fabs(div));
      if (cs[] == 1.)
        full_max = max(full_max, fabs(div));
      sum2 += sq(div) * area / cs[];
      net += div * area;
      energy += .5 * (sq(u.x[]) + sq(u.y[])) * area * cs[];
      volume += area * cs[];
    }
  }
  return (Flow){weighted_max, full_max, sqrt(sum2 / volume), net, energy, volume};
}

static coord force_coefficients(void)
{
  coord pressure_force = {0}, viscous_force = {0};
  if (cylinder)
    embed_force(p, u, dynamic_viscosity, &pressure_force, &viscous_force);
  double scale = 2. / (sq(2. * umax / 3.) * diameter);
  return (coord){scale * (pressure_force.x + viscous_force.x),
                 scale * (pressure_force.y + viscous_force.y), 0.};
}

static double pressure_difference(void)
{
  if (!cylinder)
    return interpolate(p, .4, yc) - interpolate(p, 1.8, yc);
  /* Fluid-side linear extrapolation to the two cylinder probes. Use the
     same physical probe offsets on every mesh, then report this choice. */
  double h = box / (1 << channel_level);
  double front = xc - diameter / 2., back = xc + diameter / 2.;
  return 2. * (interpolate(p, front - h, yc) - interpolate(p, back + h, yc))
       - (interpolate(p, front - 2. * h, yc) - interpolate(p, back + 2. * h, yc));
}

static double channel_profile_error(void)
{
  double error2 = 0., norm2 = 0.;
  foreach (reduction(+:error2) reduction(+:norm2)) {
    if (x > .4 && x < 1.8) {
      double exact = inlet(y);
      error2 += (sq(u.x[] - exact) + sq(u.y[])) * sq(Delta);
      norm2 += sq(exact) * sq(Delta);
    }
  }
  return sqrt(error2 / norm2);
}

static double positive(const char *text)
{
  char *end;
  errno = 0;
  double value = strtod(text, &end);
  if (errno || !*text || *end || !isfinite(value) || value <= 0.) {
    fprintf(stderr, "Invalid positive setting: %s\n", text);
    exit(2);
  }
  return value;
}

static int integer(const char *text, int low, int high)
{
  double value = positive(text);
  if (floor(value) != value || value < low || value > high)
    exit(2);
  return (int)value;
}

int main(int argc, char **argv)
{
  if (argc != 11 && argc != 13) {
    fprintf(stderr, "Usage: %s channel|steady|wake fixed|adaptive level duration dt tol eta regrid_steps sample_dt output_dir [capture_after capture_every]\n",
            argv[0]);
    return 2;
  }
  bool channel = strcmp(argv[1], "channel") == 0;
  bool steady = strcmp(argv[1], "steady") == 0;
  bool wake = strcmp(argv[1], "wake") == 0;
  adaptive = strcmp(argv[2], "adaptive") == 0;
  if ((!channel && !steady && !wake) ||
      (!adaptive && strcmp(argv[2], "fixed") != 0))
    return 2;
  cylinder = !channel;
  umax = wake ? 1.5 : .3;
  max_level = integer(argv[3], channel_level, 11);
  duration = positive(argv[4]);
  DT = positive(argv[5]);
  TOLERANCE = positive(argv[6]);
  NITERMAX = 512;
  eta = positive(argv[7]);
  adapt_interval = integer(argv[8], 1, 1024);
  sample_interval = positive(argv[9]);
  data_directory = argv[10];
  if (argc == 13) {
    capture_after = positive(argv[11]);
    capture_every = integer(argv[12], 1, 10000);
  }

  size(box);
  origin(0., 0.);
  N = 1 << max_level;
  mu = dynamic_viscosity;

  char path[4096];
  if (snprintf(path, sizeof path, "%s/series.csv", data_directory) >= sizeof path)
    return 2;
  series = fopen(path, "wx");
  if (snprintf(path, sizeof path, "%s/transfers.csv", data_directory) >= sizeof path)
    return 2;
  transfers = fopen(path, "wx");
  if (!series || !transfers) {
    perror("diagnostic output");
    return 2;
  }
  fprintf(series, "t,i,wet_leaves,allocated_cells,drag,lift,pressure_difference,relative_profile_l2,"
          "weighted_div_max,full_cell_div_max,wet_div_l2,net_flux,energy,wet_area,pressure_residual,"
          "pressure_cycles,predictor_cycles,viscous_cycles\n");
  fprintf(transfers, "t,i,refined,coarsened,wet_before,wet_after,allocation_after,"
          "raw_div_before,raw_div_after,raw_div_corrected,wet_l2_before,wet_l2_after,wet_l2_corrected,"
          "drag_before,drag_after,lift_before,lift_after,transfer_cycles,transfer_pressure_residual\n");
  elapsed_start = monotonic_time();
  run();
  if (fclose(series) || fclose(transfers))
    return 2;
  return capped_solves ? 3 : 0;
}

event init(t = 0)
{
  CFL = .6;
  /* The quadtree can merge fully masked siblings into one boundary leaf.
     Give cells beyond the outlet the outlet label so a merged corner
     block leaves the last wetted outlet face open. */
  mask(x > length ? right : y > height ? top : none);
  transfer_pressure.restriction = restriction_embed_linear;
  transfer_pressure.refine = transfer_pressure.prolongation = refine_embed_linear;
  transfer_pressure.embed_gradient = pressure_embed_gradient;
  transfer_pressure.depends = list_add(transfer_pressure.depends, cs);
  if (cylinder)
    solid(cs, fs, sq(x - xc) + sq(y - yc) - sq(diameter / 2.));
  else {
    foreach()
      cs[] = 1.;
    foreach_face()
      fs.x[] = 1.;
  }
  boundary({cs, fs});
  foreach() {
    u.x[] = cylinder ? 0. : inlet(y);
    u.y[] = 0.;
    p[] = cylinder ? 0. : 8. * nu * umax * (length - x) / sq(height);
    g.x[] = cylinder ? 0. : -8. * nu * umax / sq(height);
    g.y[] = 0.;
    transfer_pressure[] = 0.;
  }
  boundary({u, p, g, transfer_pressure});
  refresh_mesh_counts();
}

event properties(i++)
{
  foreach_face()
    dynamic_viscosity.x[] = nu * fm.x[];
}

static void audit_solve(mgstats stats, double budget, int step)
{
  if (stats.i >= NITERMAX && stats.resa > budget)
    capped_solves++;
  if (!isfinite(stats.resa) || stats.resa < 0.) {
    fprintf(stderr, "Invalid solve residual at t=%.17g, step %d\n", t, step);
    exit(3);
  }
}

static mgstats project_transfer(double step)
{
  scalar rhs[];
  foreach() {
    transfer_pressure[] = 0.;
    rhs[] = 0.;
    foreach_dimension()
      rhs[] += uf.x[1] - uf.x[];
    rhs[] /= step * Delta;
  }
  boundary({transfer_pressure});
  double scale = TOLERANCE / sq(step);
  /* At level 1 the masked channel is thinner than a cell, so its boundary
     and outlet are poorly represented on the correction grid. Keep the
     2.56/32 = 0.08 m correction grid and all finer levels. */
  mgstats solve = poisson(transfer_pressure, rhs, alpha = alpha, tolerance = scale, minlevel = 5);
  if (solve.resa <= scale) {
    foreach_face()
      uf.x[] -= step * alpha.x[] * face_gradient_x(transfer_pressure, 0);
  }
  return solve;
}

event end_timestep(i++, last)
{
  audit_solve(mgp, TOLERANCE / sq(dt), i);
  audit_solve(mgpf, TOLERANCE / sq(dt / 2.), i);
  audit_solve(mgu, TOLERANCE, i);
  Flow flow = flow_diagnostics();
  maximum_weighted_div = max(maximum_weighted_div, flow.weighted_max * dt);
  maximum_pressure_residual = max(maximum_pressure_residual, mgp.resa * sq(dt));
  double cfl = 0.;
  foreach_face (reduction(max:cfl))
    if (fm.x[] > 0.)
      cfl = max(cfl, fabs(uf.x[]) * dt / (fm.x[] * Delta));
  maximum_cfl = max(maximum_cfl, cfl);
  minimum_step = min(minimum_step, dt);
  maximum_step = max(maximum_step, dt);
  completed_steps++;
  leaf_steps += wet;
  allocated_steps += allocated;
}

event adapt(i++, last)
{
  if (!adaptive || (i + 1) % adapt_interval)
    return 0;
  long wet_before = wet;
  Flow before = flow_diagnostics();
  coord force_before = force_coefficients();
  face vector kept[];
  foreach_face()
    kept.x[] = uf.x[];
  astats change = adapt_wavelet({cs, u}, (double[]){1.e-6, eta, eta}, max_level, channel_level);
  if (!change.nf && !change.nc) {
    /* adapt_wavelet fills every halo before it decides whether to change the
       tree. An attempt that changes nothing must leave the faces as they were. */
    foreach_face()
      uf.x[] = kept.x[];
    Flow restored = flow_diagnostics();
    if (memcmp(&restored, &before, sizeof before)) {
      fprintf(stderr, "An unchanged regrid altered the flow at step %d, t=%.17g\n", i, t + dt);
      exit(3);
    }
    fprintf(stderr, "UNCHANGED_REGRID %d %.17g %.17g %.17g %.17g\n",
            i, t + dt, dt, before.weighted_max, restored.weighted_max);
    return 0;
  }
  fractions_cleanup(cs, fs);
  foreach_face()
    if (!fs.x[])
      uf.x[] = 0.;
  event("properties");
  refresh_mesh_counts();
  Flow after = flow_diagnostics();
  coord force_after = force_coefficients();

  /* With embedded boundaries, refine_face is not a solenoidal transfer.
     Measure the resulting imbalance and project the transferred face flux
     before it is used again. Keep the correction pressure separate from p. */
  mgstats correction = project_transfer(dt);
  audit_solve(correction, TOLERANCE / sq(dt), i);
  Flow corrected = flow_diagnostics();
  changes++;
  fprintf(transfers, "%.17g,%d,%d,%d,%ld,%ld,%ld,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,"
          "%.17g,%.17g,%.17g,%.17g,%d,%.17g\n",
          t + dt, i, change.nf, change.nc, wet_before, wet, allocated,
          before.weighted_max, after.weighted_max, corrected.weighted_max,
          before.wet_l2, after.wet_l2, corrected.wet_l2,
          force_before.x, force_after.x, force_before.y, force_after.y,
          correction.i, correction.resa);
  fflush(transfers);
  if (correction.resa > TOLERANCE / sq(dt)) {
    fprintf(stderr, "Transfer projection failed at step %d, t=%.17g\n", i, t + dt);
    exit(3);
  }
}

static void write_record(FILE *file, const void *data, size_t bytes)
{
  if (fwrite(data, 1, bytes, file) != bytes) {
    perror("snapshot write");
    exit(2);
  }
}

/* Physical leaves and the solver's actual face fluxes. Integer coordinates
   are relative to the L0 square at each record's level. A face's coordinate
   is the index of its low endpoint on that level along each axis. */
static void save_snapshot(int step)
{
  char path[4096];
  if (snprintf(path, sizeof path, "%s/frame-%07d.flow", data_directory, step) >= sizeof path)
    exit(2);
  FILE *file = fopen(path, "wbx");
  if (!file) {
    perror("snapshot open");
    exit(2);
  }
  long n_cells = 0, n_faces = 0;
  foreach (reduction(+:n_cells))
    n_cells++;
  foreach_face (reduction(+:n_faces))
    n_faces++;
  uint32_t header[] = {0x01020304, (uint32_t) step, (uint32_t) n_cells, (uint32_t) n_faces};
  double dimensions[] = {t, box, length, height, xc, yc, diameter};
  write_record(file, "NSFLOW01", 8);
  write_record(file, header, sizeof header);
  write_record(file, dimensions, sizeof dimensions);
  scalar omega[];
  vorticity(u, omega);
  foreach() {
    uint32_t address[] = {(uint32_t) level, (uint32_t) point.i - GHOSTS,
                          (uint32_t) point.j - GHOSTS};
    double values[] = {cs[], cs[] > 0. ? u.x[] : 0., cs[] > 0. ? u.y[] : 0.,
                       cs[] > 0. ? p[] : 0., cs[] > 0. ? omega[] : 0.};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  foreach_face (x) {
    uint32_t address[] = {0, (uint32_t) level, (uint32_t) llround(x / Delta),
                          (uint32_t) llround(y / Delta - .5)};
    double values[] = {fs.x[], uf.x[]};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  foreach_face (y) {
    uint32_t address[] = {1, (uint32_t) level, (uint32_t) llround(x / Delta - .5),
                          (uint32_t) llround(y / Delta)};
    double values[] = {fs.y[], uf.y[]};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  if (fclose(file))
    exit(2);
}

static void record_sample(int step)
{
  if (fabs(t - last_sample) < 1.e-11)
    return;
  last_sample = t;
  Flow flow = flow_diagnostics();
  coord force = force_coefficients();
  fprintf(series, "%.17g,%d,%ld,%ld,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,%.17g,"
          "%.17g,%.17g,%.17g,%d,%d,%d\n",
          t, step, wet, allocated, force.x, force.y, pressure_difference(),
          cylinder ? 0. : channel_profile_error(),
          flow.weighted_max, flow.full_cell_max, flow.wet_l2, flow.net, flow.energy, flow.volume,
          mgp.resa, mgp.i, mgpf.i, mgu.i);
  fflush(series);
  if (capture_every && t + 1.e-11 >= capture_after && observations++ % capture_every == 0)
    save_snapshot(step);
}

event observe(t = 0.; t += sample_interval; t <= duration)
{
  record_sample(i);
}

event stop(t = duration)
{
  record_sample(i);
  printf("{\"t\":%.17g,\"steps\":%ld,\"wet_leaves\":%ld,\"allocated_cells\":%ld,"
         "\"wet_leaf_steps\":%lld,\"allocated_cell_steps\":%lld,\"regrids\":%ld,"
         "\"failed_solve_caps\":%ld,\"max_weighted_volume_defect\":%.17g,"
         "\"max_scaled_pressure_residual\":%.17g,\"max_observed_face_cfl\":%.17g,"
         "\"min_dt\":%.17g,\"max_dt\":%.17g,\"elapsed_solver_s\":%.17g}\n",
         t, completed_steps, wet, allocated, leaf_steps, allocated_steps, changes, capped_solves,
         maximum_weighted_div, maximum_pressure_residual, maximum_cfl, minimum_step, maximum_step,
         monotonic_time() - elapsed_start);
  return 1;
}

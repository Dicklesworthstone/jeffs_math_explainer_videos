/* A fully specified Navier-Stokes solution for a measured adaptive-grid test.
 * A narrow, smooth layer moves upward at constant speed and spreads under
 * viscosity. Each Fourier mode is transported by V and decays by exp(-nu k²t).
 */
#include "grid/quadtree.h"
#include "navier-stokes/centered.h"
#include <time.h>
#include "flow_snapshot.h"

static const double visc = .001, crossflow = .35, end_time = .4, sample_dt = .025;
static const double mesh_error = .001;
static const int minimum_level = 4, comparison_level = 7;
static int maximum_level, changes;
static bool adaptive;
static face vector dynamic_viscosity[];
static long long leaf_steps, allocated_steps;
static double peak_error, peak_volume_error, peak_pressure_residual, largest_face_cfl, started;
static const char *capture_directory;
static const double geometry[] = {1., 1., 1., 0., 0., 0.};

static double seconds(void)
{
  struct timespec time;
  if (clock_gettime(CLOCK_MONOTONIC, &time))
    abort();
  return time.tv_sec + 1.e-9 * time.tv_nsec;
}

static double exact_u(double y, double time)
{
  double value = .3;
  double local = y - .24 - crossflow * time;
  for (int mode = 1; mode <= 18; mode++) {
    double k = 2. * pi * mode;
    value += .09 * exp(-sq(k) * (.5 * sq(.045) + visc * time)) * cos(k * local);
  }
  return value;
}

static void set_exact_initial_field(void)
{
  foreach() {
    u.x[] = exact_u(y, 0.);
    u.y[] = crossflow;
  }
  boundary({u});
}

static long allocation(void)
{
  long count = sq(tree->L[0]->len);
  for (int l = 1; l <= grid->depth; l++)
    count += 4 * tree->L[l]->nc;
  return count;
}

int main(int argc, char **argv)
{
  if (argc != 4 && argc != 5)
    return 2;
  adaptive = strcmp(argv[1], "adaptive") == 0;
  bool coarse = strcmp(argv[1], "coarse") == 0;
  if (!adaptive && !coarse && strcmp(argv[1], "fine"))
    return 2;
  DT = atof(argv[2]);
  TOLERANCE = atof(argv[3]);
  capture_directory = argc == 5 ? argv[4] : NULL;
  if (!isfinite(DT) || !isfinite(TOLERANCE) || DT <= 0. || TOLERANCE <= 0.)
    return 2;
  maximum_level = coarse ? minimum_level : comparison_level;
  size(1.);
  periodic(right);
  periodic(top);
  mu = dynamic_viscosity;
  N = 1 << maximum_level;
  printf("time,steps,cells,allocated_cells,relative_l2,weighted_divergence,kinetic_energy\n");
  started = seconds();
  run();
  return 0;
}

event init(t = 0)
{
  CFL = .6;
  set_exact_initial_field();
  if (adaptive)
    for (int pass = 0; pass < 8; pass++) {
      astats change = adapt_wavelet({u.x}, (double[]){mesh_error}, maximum_level, minimum_level);
      if (!change.nf && !change.nc)
        break;
      set_exact_initial_field();
    }
}

event properties(i++)
{
  foreach_face()
    dynamic_viscosity.x[] = visc * fm.x[];
}

event end_timestep(i++, last)
{
  if (mgp.resa * sq(dt) > TOLERANCE || mgpf.resa * sq(dt / 2.) > TOLERANCE
      || mgu.i == 0 || mgu.resa > TOLERANCE) {
    fprintf(stderr, "Shear solve failed at step %d\n", i);
    exit(3);
  }
  double defect = 0., cfl = 0.;
  foreach (reduction(max:defect))
    defect = max(defect, fabs(uf.x[1] - uf.x[] + uf.y[0,1] - uf.y[]) * dt / Delta);
  foreach_face (reduction(max:cfl))
    cfl = max(cfl, fabs(uf.x[]) * dt / Delta);
  peak_volume_error = max(peak_volume_error, defect);
  peak_pressure_residual = max(peak_pressure_residual, mgp.resa * sq(dt));
  largest_face_cfl = max(largest_face_cfl, cfl);
  leaf_steps += grid->n;
  allocated_steps += allocation();
}

event adapt(i++, last)
{
  if (adaptive && (i + 1) % 4 == 0) {
    astats change = adapt_wavelet({u.x}, (double[]){mesh_error}, maximum_level, minimum_level);
    changes += change.nf > 0 || change.nc > 0;
  }
}

static void record_observation(double at_time, int step)
{
  double error2 = 0., norm2 = 0., divergence = 0., energy = 0.;
  foreach (reduction(+:error2) reduction(+:norm2) reduction(max:divergence) reduction(+:energy)) {
    double ux = exact_u(y, at_time);
    /* The squared error and reference norm use each cell's physical area. */
    error2 += (sq(u.x[] - ux) + sq(u.y[] - crossflow)) * sq(Delta);
    norm2 += (sq(ux) + sq(crossflow)) * sq(Delta);
    divergence = max(divergence, fabs(uf.x[1] - uf.x[] + uf.y[0,1] - uf.y[]) / Delta);
    energy += .5 * (sq(u.x[]) + sq(u.y[])) * sq(Delta);
  }
  double relative_error = sqrt(error2 / norm2);
  if (!isfinite(relative_error) || !isfinite(divergence))
    exit(3);
  peak_error = max(peak_error, relative_error);
  printf("%.17g,%d,%ld,%ld,%.17g,%.17g,%.17g\n",
         at_time, step, grid->n, allocation(), relative_error, divergence, energy);
  if (capture_directory)
    write_flow_snapshot(capture_directory, step, geometry);
}

event observations(t = 0.; t += sample_dt; t < end_time - 1.e-10)
{
  record_observation(t, i);
}

event finished(t = end_time)
{
  record_observation(t, i);
  printf("{\"end\":%.17g,\"steps\":%d,\"cells\":%ld,\"allocated_cells\":%ld,"
         "\"leaf_steps\":%lld,\"allocated_cell_steps\":%lld,\"regrids\":%d,"
         "\"peak_rel_l2\":%.17g,\"peak_volume_defect\":%.17g,"
         "\"peak_scaled_pressure_residual\":%.17g,\"peak_cfl\":%.17g,"
         "\"solver_seconds\":%.17g}\n",
         t, i, grid->n, allocation(), leaf_steps, allocated_steps, changes,
         peak_error, peak_volume_error, peak_pressure_residual, largest_face_cfl, seconds() - started);
  return 1;
}

/* NSFLOW01: little-endian header, physical leaves and actual face fluxes.
 * Pressure, centered velocity and vorticity are kept separately from flux.
 */
#include <stdint.h>

static void write_record(FILE *file, const void *data, size_t bytes)
{
  if (fwrite(data, 1, bytes, file) != bytes) {
    perror("snapshot write");
    exit(2);
  }
}

static void write_flow_snapshot(const char *directory, int step,
                                const double geometry[6])
{
  char path[4096];
  if (snprintf(path, sizeof path, "%s/frame-%07d.flow", directory, step) >= sizeof path)
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
  write_record(file, "NSFLOW01", 8);
  write_record(file, header, sizeof header);
  write_record(file, &t, sizeof t);
  write_record(file, geometry, 6 * sizeof(double));
  scalar omega[];
  vorticity(u, omega);
  foreach() {
    uint32_t address[] = {(uint32_t) level, (uint32_t) point.i - GHOSTS,
                          (uint32_t) point.j - GHOSTS};
    double values[] = {cm[], cm[] > 0. ? u.x[] : 0., cm[] > 0. ? u.y[] : 0.,
                       cm[] > 0. ? p[] : 0., cm[] > 0. ? omega[] : 0.};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  foreach_face (x) {
    uint32_t address[] = {0, (uint32_t) level, (uint32_t) llround(x / Delta),
                          (uint32_t) llround(y / Delta - .5)};
    double values[] = {fm.x[], uf.x[]};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  foreach_face (y) {
    uint32_t address[] = {1, (uint32_t) level, (uint32_t) llround(x / Delta - .5),
                          (uint32_t) llround(y / Delta)};
    double values[] = {fm.y[], uf.y[]};
    write_record(file, address, sizeof address);
    write_record(file, values, sizeof values);
  }
  if (fclose(file)) {
    exit(2);
  }
}

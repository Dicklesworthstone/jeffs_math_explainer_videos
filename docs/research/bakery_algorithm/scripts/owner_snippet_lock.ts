class BakeryLock {
  entering: boolean[] = [];
  tickets: number[] = [];
  numProcesses = 3;
  lock(i: number) {
    this.entering[i] = true;
    this.tickets[i] = Math.max(...this.tickets) + 1;
    this.entering[i] = false;
    for (let j = 0; j < this.numProcesses; j++) {
      while (this.entering[j]) {}
      while (this.tickets[j] !== 0 && (this.tickets[j], j) < (this.tickets[i], i)) {}
    }
  }
}

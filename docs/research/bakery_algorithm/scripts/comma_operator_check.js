// Reproduces the memo's claim that the article's TypeScript condition
//   (this.tickets[j], j) < (this.tickets[i], i)
// uses JavaScript's comma operator and therefore evaluates to (j < i), not a
// lexicographic (ticket, id) comparison.   Run:  node comma_operator_check.js
const t = [0, 5, 3];
const i = 2, j = 1;
const asWritten = (t[j], j) < (t[i], i);            // comma operator => j < i
const lexicographic = t[j] < t[i] || (t[j] === t[i] && j < i);
console.log("(t[j], j) < (t[i], i) evaluates to", asWritten, "| j < i =", j < i,
            "| lexicographic would be", lexicographic);
// Expected output: (t[j], j) < (t[i], i) evaluates to true | j < i = true | lexicographic would be false

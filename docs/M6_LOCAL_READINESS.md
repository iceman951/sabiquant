# Local M6 numerical foundations

## Authority and scope

The owner authorized continuous local work with the broadest meaningful test
coverage feasible. M5 corrections are saved at `018b14c`. This authorizes the
agent to stage M6 locally, make implementation decisions explicit and collect
evidence. It does not record formal M5 acceptance or owner API approval.
Primary-platform native CI, source/maintenance and release gates remain open;
publication and remote writes remain disabled.

M6 retains the roadmap scope: bisection, Brent and safeguarded Newton;
linear, log-linear, natural cubic and a specifically named monotone cubic
Hermite interpolator; normal density, distribution and inverse; compensated
summation. Research is separate from implementation and measured accuracy.
No family is marked complete from a research document or narrow passing test.

All library algorithms use only Rust std, finite explicit inputs, checked
results and typed contextual errors. Algorithms have bounded work; user
callbacks retain responsibility for their mathematical continuity, accuracy
and any panic. Immutable data values have private invariant-carrying fields.
There is no global configuration, I/O, unsafe code or copied reference code.

## Summation decision

`compensated_sum(&[f64])` implements an original Neumaier compensated
reduction in supplied order. All inputs are checked before arithmetic. Every
partial sum, accumulated correction and final result must remain finite;
an overflow is a typed error even if later cancellation could recover a finite
exact result. Empty sums and exact zero return positive zero. Compensation
improves common reductions; it does not guarantee a correctly rounded answer
or invariance to reordering.

`exact_sum(&[f64])` provides the stronger alternative for severe
cancellation and intermediate-overflow cases. Every finite binary64 is an
integer multiple of `2^-1074`. Separate positive and negative fixed-width
integer accumulators retain those integers, subtract exactly, then round once
to binary64, nearest with ties to even. The accumulator capacity includes the
full binary64 exponent range plus the slice-length bit width. Only a final
rounded overflow is an error; subnormal sums are exact. This is an original
simple implementation of the general fixed-point superaccumulator idea, not
Neal's overlapping-chunk algorithm or a translation of its code.

Sources inspected on 2026-10-12: Neumaier's original
[1974 paper](https://arnold-neumaier.at/scan/01.pdf) (author-hosted original
scan, p. 40, method IV, equation IV.1; the two correction branches were
visually inspected against the independent implementation), and
Neal's [2015 author-hosted paper](https://www.cs.toronto.edu/~radford/ftp/xsum.pdf),
especially pp. 3–4 on representation and fixed-point capacity. No performance
claim from that paper transfers to this implementation. Exact rational Python
oracles independently check rounding, carry/borrow, every exponent,
subnormal/normal transitions, overflow ties, permutations and cancellation.

## Remaining numerical contracts

The agent adopts the explicit local contracts in
[solver research](M6_SOLVER_RESEARCH.md),
[interpolation research](M6_INTERPOLATION_RESEARCH.md) and
[normal research](M6_NORMAL_RESEARCH.md). The algorithms are implemented
locally; aggregate measured evidence and remaining gates are in
[M6_NUMERICAL_VALIDATION](M6_NUMERICAL_VALIDATION.md). Solver
budgets count every function and derivative call. Interpolation pins distinct
natural/PCHIP rules and explicit extrapolation, retaining exact stored knots.
Normal inverse uses centered or logarithmic residuals to retain median
neighbors and subnormal probabilities. Increasing-precision oracle agreement supplies empirical
evidence, not an interval certificate; Rust elementary functions retain their
unspecified-precision and platform qualifications.

# Sinsemilla invocations

## $\text{Commit}^\text{ivk}$

([Specification](https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit))

Our goal is to constrain a [Sinsemilla $\textsf{ShortCommit}$](sinsemilla.md#use-as-a-commitment-scheme)
over the message

$$\ItoLEBSP{\BaseLength{Orchard}}(\AuthSignPublic) \bconcat \ItoLEBSP{\BaseLength{Orchard}}(\NullifierKey)$$

where each of $\AuthSignPublic$, $\NullifierKey$ is a field element (255 bits). Sinsemilla
operates on multiples of 10 bits, so we start by decomposing the message into chunks:

$$
\begin{align}
\ItoLEBSP{\BaseLength{Orchard}}(\AuthSignPublic) &= a \bconcat b_0 \bconcat b_1 \\
  &= (\text{bits 0..=249 of } \AuthSignPublic) \bconcat
     (\text{bits 250..=253 of } \AuthSignPublic) \bconcat
     (\text{bit 254 of } \AuthSignPublic)  \\
\ItoLEBSP{\BaseLength{Orchard}}(\NullifierKey) &= b_2 \bconcat c \bconcat d_0 \bconcat d_1 \\
  &= (\text{bits 0..=4 of } \NullifierKey) \bconcat
     (\text{bits 5..=244 of } \NullifierKey) \bconcat
     (\text{bits 245..=253 of } \NullifierKey) \bconcat
     (\text{bit 254 of } \NullifierKey) \\
\end{align}
$$

Then we recompose the chunks into message pieces:

$$
\begin{array}{|c|l|}
\hline
\text{Length (bits)} & \text{Piece} \\\hline
250 & a \\
10  & b = b_0 \bconcat b_1 \bconcat b_2 \\
240 & c \\
10  & d = d_0 \bconcat d_1 \\\hline
\end{array}
$$

Each message piece is constrained by Sinsemilla to its stated length. Additionally,
$\AuthSignPublic$ and $\NullifierKey$ are witnessed as field elements, so we know they are
canonical. However, we need additional constraints to enforce that:

- The chunks are the correct bit lengths (or else they could overlap in the decompositions
  and allow the prover to witness an arbitrary Sinsemilla $\textsf{ShortCommit}$ message).
- The chunks contain the canonical decompositions of $\AuthSignPublic$ and $\NullifierKey$
  (or else the prover could witness an input to Sinsemilla $\textsf{ShortCommit}$ that is
  equivalent to $\AuthSignPublic$ and $\NullifierKey$ but not identical).

### Bit length constraints

Chunks $a$ and $c$ are directly constrained by Sinsemilla. For the remaining chunks, we
use the following constraints:

- Three [short lookup range checks](lookup_range_check.md#short-range-check) on:
  - $b_0$ (4 bits)
  - $b_2$ (5 bits)
  - $d_0$ (9 bits)
- Two boolean constraints:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_\text{canon} \cdot \texttt{bool\_check}(b_1) = 0 \\\hline
3 & q_\text{canon} \cdot \texttt{bool\_check}(d_1) = 0 \\\hline
\end{array}
$$

### Decomposition constraints

Now that all chunks are constrained to the correct bit lengths, we can constrain the
decompositions in the obvious way:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_\text{canon} \cdot (b - (b_0 + b_1 \cdot 2^4 + b_2 \cdot 2^5)) = 0 \\\hline
2 & q_\text{canon} \cdot (d - (d_0 + d_1 \cdot 2^9)) = 0 \\\hline
2 & q_\text{canon} \cdot (a + b_0 \cdot 2^{250} + b_1 \cdot 2^{254} - \textsf{ak}) = 0 \\\hline
2 & q_\text{canon} \cdot (b_2 + c \cdot 2^5 + d_0 \cdot 2^{245} + d_1 \cdot 2^{254} - \textsf{nk}) = 0 \\\hline
\end{array}
$$

### Canonicity checks for $\AuthSignPublic$

At this point, we have constrained $\AuthSignPublic$ to be a 255-bit value, with top bit
$b_1$. The Pallas base field modulus has the form $q_\mathbb{P} = 2^{254} + t_\mathbb{P}$,
where $t_\mathbb{P} = \mathtt{0x224698fc094cf91b992d30ed00000001}$ is 126 bits.

If the top bit is not set, then the remaining bits will always comprise a canonical
encoding of a field element. Thus the $\AuthSignPublic$ canonicity checks below are
enforced if and only if $b_1 = 1$.

If the top bit is set, we need to enforce that $b_0 = 0$, and $a < t_\mathbb{P}$. To check
the latter, we use a base-$2^{10}$ variant of the method used in libsnark (originally from
[[SVPBABW2012](https://eprint.iacr.org/2012/598.pdf), Appendix C.1]), i.e.:

- Let $t'$ be the smallest power of $2^{10}$ greater than $t_\mathbb{P}$.
  - For Pallas, $t' = 2^{130}$.
- Enforce $0 \leq a < t'$.
  - We can do this for free because the Sinsemilla gadget is already decomposing $a$: we
    constrain the running sum element
    $z_{a,13} = \left(a - \sum\limits_{i=0}^{12} 2^{10i} \cdot \mathbf{a}_i\right)/2^{130}$
    to be zero.
- Let $a' = a + t' - t_\mathbb{P}$, and enforce $0 \leq a' < t'$.
  - This requires a running sum with 13 lookups, the end of which we constrain to be zero.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_\text{canon} \cdot b_1 \cdot b_0 = 0 \\\hline
3 & q_\text{canon} \cdot b_1 \cdot z_{a,13} = 0 \\\hline
2 & q_\text{canon} \cdot (a + 2^{130} - t_\mathbb{P} - a') = 0 \\\hline
3 & q_\text{canon} \cdot b_1 \cdot z_{a',13} = 0 \\\hline
\end{array}
$$

### Canonicity checks for $\NullifierKey$

The $\NullifierKey$ canonicity checks are similar to the $\AuthSignPublic$ checks, except
that now if $d_1 = 1$ we need to enforce $d_0 = 0$ and $b_2 + c \cdot 2^5 < t_\mathbb{P}$.

$b_2$ is constrained to be 5 bits, so we constrain the running sum element $z_{c,13}$ to
be zero, which ensures that $b_2 + c \cdot 2^5$ is 135 bits. Then we use the same method
as for $\AuthSignPublic$ but setting $t' = 2^{140}$, which requires a running sum with
14 lookups.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_\text{canon} \cdot d_1 \cdot d_0 = 0 \\\hline
3 & q_\text{canon} \cdot d_1 \cdot z_{c,13} = 0 \\\hline
2 & q_\text{canon} \cdot (b_2 + c \cdot 2^5 + 2^{140} - t_\mathbb{P} - {b_2}c') = 0 \\\hline
3 & q_\text{canon} \cdot d_1 \cdot z_{{b_2}c',14} = 0 \\\hline
\end{array}
$$

### Region layout

The constraints controlled by the $q_\text{canon}$ selector are arranged across all 10
advice columns, requiring two rows.

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c}
    &          &    &           &                 &               &     &     &     &     & q_\text{canon} \\\hline
a   & b        & c  & d         & \AuthSignPublic & \NullifierKey & b_0 & b_1 & b_2 & d_0 & 0 \\\hline
d_1 & z_{a,13} & a' & z_{a',13} & z_{c,13}        & {b_2}c'       & z_{{b_2}c',14}  & & & & 1 \\\hline
\end{array}
$$

str4d's preferred layout, arranged across 9 advice columns and 2 rows:

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c}
                &   &   &     &     &     &          &         &                & q_\text{canon} \\\hline
\AuthSignPublic & a & b & b_0 & b_1 & b_2 & z_{a,13} & a'      & z_{a',13}      & 0 \\\hline
\NullifierKey   & c & d & d_0 & d_1 &     & z_{c,13} & {b_2}c' & z_{{b_2}c',14} & 1 \\\hline
\end{array}
$$

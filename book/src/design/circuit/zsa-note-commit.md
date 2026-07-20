# OrchardZSA NoteCommit

In the OrchardZSA protocol, the $\NoteCommit$ function is defined as follows:
$$
\begin{align}
&\mathsf{NoteCommit^{OrchardZSA}_{rcm}}(
\DiversifiedTransmitBaseRepr,
\DiversifiedTransmitPublicRepr,
\ItoLEBSP{64}(\mathsf{v}),
\ItoLEBSP{\BaseLength{Orchard}}(\rho),
\ItoLEBSP{\BaseLength{Orchard}}(\psi),
\mathsf{AssetBase}) = \\
&\;\;\;\;\;
\begin{cases}
\mathsf{h_{ZEC}} + [\mathsf{rcm}]\,\mathsf{GroupHash}^{\mathbb{P}}(\texttt{"z.cash:Orchard-NoteCommit-r"}, \texttt{""}) &\text{ if } \mathsf{AssetBase} = \mathcal{V}^{\mathsf{Orchard}} \\
\mathsf{h_{ZSA}} + [\mathsf{rcm}]\,\mathsf{GroupHash}^{\mathbb{P}}(\texttt{"z.cash:Orchard-NoteCommit-r"}, \texttt{""}) &\text{otherwise}
\end{cases}
\end{align}
$$
where
$$
\begin{align}
\mathsf{h_{ZEC}} =&\;\;\mathsf{SinsemillaHashToPoint}(\texttt{"z.cash:Orchard-NoteCommit-M"}, \\
&\;\;\;\;\;
\DiversifiedTransmitBaseRepr \bconcat
\DiversifiedTransmitPublicRepr \bconcat
\ItoLEBSP{64}(\mathsf{v}) \bconcat
\ItoLEBSP{\BaseLength{Orchard}}(\rho) \bconcat
\ItoLEBSP{\BaseLength{Orchard}}(\psi))
\end{align}
$$
and
$$
\begin{align}
\mathsf{h_{ZSA}} =&\;\;\mathsf{SinsemillaHashToPoint}(\texttt{"z.cash:ZSA-NoteCommit-M"}, \\
&\;\;\;\;\;
\DiversifiedTransmitBaseRepr \bconcat
\DiversifiedTransmitPublicRepr \bconcat
\ItoLEBSP{64}(\mathsf{v}) \bconcat
\ItoLEBSP{\BaseLength{Orchard}}(\rho) \bconcat
\ItoLEBSP{\BaseLength{Orchard}}(\psi) \bconcat
\mathsf{AssetBase})
\end{align}
$$
where:
- $\DiversifiedTransmitBaseRepr, \DiversifiedTransmitPublicRepr, \mathsf{AssetBase}$ are representations of
  Pallas curve points, with $255$ bits used for the $x$-coordinate and $1$ bit used for
  the $y$-coordinate.
- $\rho, \psi$ are Pallas base field elements.
- $\mathsf{v}$ is a $64$-bit value.
- $\BaseLength{Orchard} = 255.$

We will only present the updates and additions compared to
[Orchard $\NoteCommit$](./note-commit.md).

## Message decomposition

Sinsemilla operates on multiples of 10 bits, so we start by decomposing the message into
chunks:

$$
\begin{aligned}
\DiversifiedTransmitBaseRepr &= a \bconcat b_0 \bconcat b_1 \bconcat b_2 \\
 &= (\text{bits 0..=249 of } x(\mathsf{g_d})) \bconcat
    (\text{bits 250..=253 of } x(\mathsf{g_d})) \bconcat
    (\text{bit 254 of } x(\mathsf{g_d})) \bconcat
    (ỹ \text{ bit of } \mathsf{g_d}) \\
\DiversifiedTransmitPublicRepr &= b_3 \bconcat c \bconcat d_0 \bconcat d_1 \\
 &= (\text{bits 0..=3 of } x(\mathsf{pk_d})) \bconcat
    (\text{bits 4..=253 of } x(\mathsf{pk_d})) \bconcat
    (\text{bit 254 of } x(\mathsf{pk_d})) \bconcat
    (ỹ \text{ bit of } \mathsf{pk_d}) \\
\ItoLEBSP{64}(v) &= d_2 \bconcat d_3 \bconcat e_0 \\
 &= (\text{bits 0..=7 of } v) \bconcat
    (\text{bits 8..=57 of } v) \bconcat
    (\text{bits 58..=63 of } v) \\
\ItoLEBSP{\BaseLength{Orchard}}(\rho) &= e_1 \bconcat f \bconcat g_0 \\
 &= (\text{bits 0..=3 of } \rho) \bconcat
    (\text{bits 4..=253 of } \rho) \bconcat
    (\text{bit 254 of } \rho) \\
\ItoLEBSP{\BaseLength{Orchard}}(\psi) &= g_1 \bconcat g_2 \bconcat h_0 \bconcat h_1 \\
 &= (\text{bits 0..=8 of } \psi) \bconcat
    (\text{bits 9..=248 of } \psi) \bconcat
    (\text{bits 249..=253 of } \psi) \bconcat
    (\text{bit 254 of } \psi) \\
\mathsf{AssetBase} &= h_2^{ZSA} \bconcat i \bconcat j_0 \bconcat j_1 \\
 &= (\text{bits 0..=3 of } x(\mathsf{AssetBase})) \bconcat
    (\text{bits 4..=253 of } x(\mathsf{AssetBase})) \bconcat
    (\text{bit 254 of } x(\mathsf{AssetBase})) \bconcat
    (ỹ \text{ bit of } \mathsf{AssetBase}) \\
\end{aligned}
$$

Then we recompose the chunks into message pieces:

$$
\begin{array}{|c|l|}
\hline
\text{Length (bits)} & \text{Piece} \\\hline
250 & a \\
 10 & b = b_0 \bconcat b_1 \bconcat b_2 \bconcat b_3 \\
250 & c \\
 60 & d = d_0 \bconcat d_1 \bconcat d_2 \bconcat d_3 \\
 10 & e = e_0 \bconcat e_1 \\
250 & f \\
250 & g = g_0 \bconcat g_1 \bconcat g_2 \\
 10 & h = h_0 \bconcat h_1 \bconcat h_2^{ZSA} \\
250 & i \\
 10 & j_0 \bconcat j_1 \bconcat j_2 \\
 \hline
\end{array}
$$

where $j_2$ is 8 zero bits (corresponding to the padding applied by the Sinsemilla
[$\mathsf{pad}$ function](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash)).

## Message piece decomposition

In this section, we will only present the updates and additions compared to
[Message piece decomposition for Orchard](./note-commit.md#message-piece-decomposition).

As $a$, $c$ and $f$, $i$ is also witnessed and constrained outside the gate.

### $h$ decomposition

$$\begin{cases}
h^{ZEC} = h_0 \bconcat h_1 \bconcat 0000 \\
h^{ZSA} = h_0 \bconcat h_1 \bconcat h_2^{ZSA}
\end{cases}$$
$h^{ZEC}$ and $h^{ZSA}$ have been constrained to be $10$ bits each by the Sinsemilla hash.

#### Region layout
$$
\begin{array}{|c|c|c|c|}
\hline
 A_6     & A_7       & A_8 & q_{\NoteCommit,h} \\\hline
 h^{ZEC} & h_0       & h_1 &       1           \\\hline
 h^{ZSA} & h_2^{ZSA} &     &       0           \\\hline
\end{array}
$$

#### Constraints
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint}                           \\\hline
       3      & q_{\NoteCommit,h} \cdot \BoolCheck{h_1} = 0 \\\hline
       2      & q_{\NoteCommit,h} \cdot (h^{ZEC} - (h_0 + h_1 \cdot 2^5)) = 0 \\\hline
       2      & q_{\NoteCommit,h} \cdot (h^{ZSA} - (h_0 + h_1 \cdot 2^5 + h_2^{ZSA} \cdot 2^6)) = 0 \\\hline
\end{array}
$$

Outside this gate, we have constrained:
- $\ShortLookupRangeCheck{h_0, 5}$
- $\ShortLookupRangeCheck{h_2^{ZSA}, 4}$

### $j$ decomposition

$$j = j_0 \bconcat j_1 \bconcat j_2$$
with $j_2$ is 8 zero bits.

$j$ has been constrained to be $10$ bits by the $\SinsemillaHash$.

#### Region layout
$$
\begin{array}{|c|c|c|c|}
\hline
 A_6 & A_7 & A_8 & q_{\NoteCommit,j} \\\hline
  j  & j_0 & j_1 &       1           \\\hline
\end{array}
$$

#### Constraints
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint}                           \\\hline
      3       & q_{\NoteCommit,j} \cdot \BoolCheck{j_0} = 0 \\\hline
      3       & q_{\NoteCommit,j} \cdot \BoolCheck{j_1} = 0 \\\hline
2 & q_{\NoteCommit,j} \cdot (j - (j_0 + j_1 \cdot 2)) = 0 \\\hline
\end{array}
$$

## Field element checks

In addition to [field element checks for Orchard NoteCommit](./note-commit.md#field-element-checks),
we have to
- constrain $\ItoLEBSP{\BaseLength{Orchard}}(x(\mathsf{AssetBase}))$
  to be 255-bit value, with top bit $j_0$.
- constrain $\ItoLEBSP{\BaseLength{Orchard}}(x(\mathsf{AssetBase})) = x(\mathsf{AssetBase}) \pmod{q_\mathbb{P}}$
where $q_\mathbb{P}$ is the Pallas base field modulus.
- check that $\mathsf{AssetBase}$ is indeed canonically-encoded field elements, i.e.
$$\ItoLEBSP{\BaseLength{Orchard}}(x(\mathsf{AssetBase})) < q_\mathbb{P}$$

$\mathsf{pk_d}$ and $\mathsf{AssetBase}$ have a similar message piece decomposition.
Thus, we will reuse the $\mathsf{pk_d}$ gates to check those constraints on $\mathsf{AssetBase}$.

### $x(\mathsf{AssetBase})$ is a Pallas base field element
Recall that $x(\mathsf{AssetBase}) = h_2^{ZSA} + 2^4 \cdot i + 2^{254} \cdot j_0$.

#### Region layout
$$
\begin{array}{|c|c|c|c|c|}
\hline
      A_6             & A_7       &  A_8  &      A_9     & q_{\NoteCommit,x(\mathsf{pk_d})} \\\hline
x(\mathsf{AssetBase}) & h_2^{ZSA} &   i   & z_{i,13}     &                1                 \\\hline
                      & j_0       & h_2i' & z_{h_2i',14} &                0                 \\\hline
\end{array}
$$

where
- $z_{i,13}$ is the index-13 running sum output by $\SinsemillaHash(i)$,
- $h_2i' = h_2^{ZSA} + 2^4 \cdot i + 2^{140} - t_\mathbb{P}$, and
- $z_{h_2i',14}$ is the index-14 running sum output by $\SinsemillaHash(h_2i')$,

#### Constraints
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_{\NoteCommit,x(\mathsf{AssetBast})} \cdot \left(h_2^{ZSA} + i \cdot 2^4 + j_0 \cdot 2^{254} - x(\mathsf{AssetBase}) \right) = 0 \\\hline
3 & q_{\NoteCommit,x(\mathsf{AssetBase})} \cdot j_0 \cdot z_{i,13} = 0 \\\hline
2 & q_{\NoteCommit,x(\mathsf{AssetBase})} \cdot (h_2^{ZSA} + i \cdot 2^4 + 2^{140} - t_\mathbb{P} - {h_2}i') = 0 \\\hline
3 & q_{\NoteCommit,x(\mathsf{AssetBase})} \cdot j_0 \cdot z_{{h_2}i',14} = 0 \\\hline
\end{array}
$$

### Decomposition of $y(\mathsf{AssetBase})$

We would like to verify that the following decomposition of $y(\mathsf{AssetBase})$ is correct
$$
\begin{align}
y &= \textsf{LSB} \bconcat k_0 \bconcat k_1 \bconcat k_2 \bconcat k_3\\
  &= \textsf{LSB}
      \bconcat \text{ (bits $1..=9$ of $y$) }
      \bconcat \text{ (bits $10..=249$ of $y$) }
      \bconcat \text{ (bits $250..=253$ of $y$) }
      \bconcat \text{ (bit $254$ of $y$) },
\end{align}
$$
where $\textsf{LSB}=j_1$.

To achieve this, we will use the same gate that is utilized to verify the correctness
of the decompositions of $y(\mathsf{g_d})$ and $y(\mathsf{pk_d})$.

Let $j = \textsf{LSB} + 2 \cdot k_0 + 2^{10} \cdot k_1$.
We decompose $j$ to be $250$ bits using a strict $25-$word
[ten-bit lookup](../decomposition.md#lookup-decomposition).
The running sum outputs allow us to susbstitute $k_1 = z_{j, 1}.$

Recall that $\textsf{LSB} = ỹ(\mathsf{AssetBase})$ was piece input to the
Sinsemilla hash and have already been boolean-constrained.
$k_0$ and $k_2$ are constrained outside this gate to $9$ and $4$ bits respectively.
To constrain the remaining chunks, we use the following constraints:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_{\NoteCommit,y} \cdot \BoolCheck{k_3} = 0 \\\hline
\end{array}
$$

Then, to check that the decomposition was correct:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_{\NoteCommit,y} \cdot \left(j - (\textsf{LSB} + k_0 \cdot 2 + k_1 \cdot 2^{10}) \right) = 0 \\\hline
2 & q_{\NoteCommit,y} \cdot \left(y - (j + k_2 \cdot 2^{250} + k_3 \cdot 2^{254}) \right) = 0 \\\hline
\end{array}
$$

### $y(\mathsf{AssetBase})$ is a Pallas base field element
Recall that
$$
\begin{align}
y &= ỹ \bconcat k_0 \bconcat k_1 \bconcat k_2 \bconcat k_3\\
  &= ỹ
      \bconcat \text{ (bits $1..=9$ of $y$) }
      \bconcat \text{ (bits $10..=249$ of $y$) }
      \bconcat \text{ (bits $250..=253$ of $y$) }
      \bconcat \text{ (bit $254$ of $y$) },
\end{align}
$$
Let $\begin{cases}
j = \textsf{LSB} + 2 \cdot k_0 + 2^{10} \cdot k_1 \\
j' = j + 2^{130} - t_\mathbb{P}
\end{cases}$.

#### Region layout
$$
\begin{array}{|c|c|c|c|c|c|}
\hline
A_5  & A_6  &   A_7    & A_8  &    A_9    & q_{\NoteCommit,y} \\\hline
 y   &  ỹ   &   k_0    & k_2  &    k_3    &         1         \\\hline
 j   & k_1  & z_{j,13} & j'   & z_{j',13} &         0         \\\hline
\end{array}
$$
where $z_{j,13}$ (resp. $z_{j', 13}$) is the index-13 running sum output by the $10$-bit lookup
decomposition of $j$ (resp. $j'$).

#### Constraints
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_{\NoteCommit,y} \cdot k_3 \cdot k_2 = 0 \\\hline
3 & q_{\NoteCommit,y} \cdot k_3 \cdot z_{j,13} = 0 \\\hline
2 & q_{\NoteCommit,y} \cdot (j + 2^{130} - t_\mathbb{P} - j') = 0 \\\hline
3 & q_{\NoteCommit,y} \cdot k_3 \cdot z_{j',13} = 0 \\\hline
\end{array}
$$

Outside this gate, we have constrained:
- $\ShortLookupRangeCheck{k_0, 9}$
- $\ShortLookupRangeCheck{k_2, 4}$

## $\mathsf{SinsemillaHashToPoint}$ evaluations
To evaluate our OrchardZSA $\NoteCommit$, we must evalute
$\mathsf{h} = \begin{cases} \mathsf{h_{ZEC}} &\text{ if } \mathsf{AssetBase} = \mathcal{V}^{\mathsf{Orchard}} \\
\mathsf{h_{ZSA}} &\text{ otherwise} \end{cases}$.

We observe that the messages to hash in $\mathsf{h_{ZEC}}$ and $\mathsf{h_{ZSA}}$  have a similar prefix
$$\begin{cases}
\mathsf{h_{ZEC}} = \mathsf{SinsemillaHashToPoint}(\texttt{"z.cash:Orchard-NoteCommit-M"},
\mathsf{prefix} \bconcat \mathsf{suffix_{ZEC}}) \\
\mathsf{h_{ZSA}} = \mathsf{SinsemillaHashToPoint}(\texttt{"z.cash:ZSA-NoteCommit-M"},
\mathsf{prefix} \bconcat \mathsf{suffix_{ZSA}})
\end{cases}$$
where
$$\begin{cases}
\mathsf{prefix} = a \bconcat b \bconcat c \bconcat d \bconcat e \bconcat f \bconcat g \\
\mathsf{suffix_{ZEC}} = h_0 \bconcat h_1 \bconcat 0000 \\
\mathsf{suffix_{ZSA}} = h_0 \bconcat h_1 \bconcat h_2^{ZSA} \bconcat i \bconcat j
\end{cases}$$

We will optimize the circuit by leveraging the fact that their respective messages share
a common prefix. More precisely, the circuit will contain the following steps:
1. Evaluate $Q_{init}$ the initial state of the hash function thanks to a multiplexer gate
$$ \mathcal{Q}_{init} = MUX(\mathcal{Q}_{ZSA}, \mathcal{Q}_{ZEC}, \mathsf{AssetBase} == \mathcal{V}^{\mathsf{Orchard}})$$
where
$$\begin{cases}
\mathcal{Q}_{ZEC} = \mathcal{Q}(\texttt{"z.cash:Orchard-NoteCommit-M"}) \\
\mathcal{Q}_{ZSA} = \mathcal{Q}(\texttt{"z.cash:ZSA-NoteCommit-M"})
\end{cases}$$
2. Evaluate $\mathsf{h_{common}}$ which is equal to the Sinsemilla hash evaluation from the inital
point $\mathcal{Q}_{init}$ with the message $\mathsf{prefix}$
$$\mathsf{h_{common}} = \mathsf{SinsemillaHash}(\mathcal{Q}_{init}, \mathsf{prefix})$$
3. Evaluate $\mathsf{h_{ZSA}}$ and $\mathsf{h_{ZEC}}$ which are equal to the Sinsemilla hash evaluations from the initial point $h_{common}$ with the message $\mathsf{suffix_{ZSA}}$ and $\mathsf{suffix_{ZEC}}$ respectively
$$\begin{cases}
\mathsf{h_{ZEC}} = \mathsf{SinsemillaHash}(\mathsf{h_{common}}, \mathsf{suffix_{ZEC}}) \\
\mathsf{h_{ZSA}} = \mathsf{SinsemillaHash}(\mathsf{h_{common}}, \mathsf{suffix_{ZSA}})
\end{cases}$$
4. Select the desired hash thanks to a multiplexer gate
$$ \mathsf{h} = MUX(\mathsf{h_{ZSA}}, \mathsf{h_{ZEC}}, \mathsf{AssetBase} == \mathcal{V}^{\mathsf{Orchard}})$$

![](./zsa-note-commit.png)


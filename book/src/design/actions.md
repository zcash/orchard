# Actions

In Sprout, we had a single proof that represented two spent notes and two new notes. This
was necessary in order to facilitate spending multiple notes in a single transaction (to
balance value, an output of one JoinSplit could be spent in the next one), but also
provided a minimal level of arity-hiding: single-JoinSplit transactions all looked like
2-in 2-out transactions, and in multi-JoinSplit transactions each JoinSplit looked like a
1-in 1-out.

In Sapling, we switched to using value commitments to balance the transaction, removing
the min-2 arity requirement. We opted for one proof per spent note and one (much simpler)
proof per output note, which greatly improved the performance of generating outputs, but
removed any arity-hiding from the proofs (instead having the transaction builder pad
transactions to 1-in, 2-out).

For Orchard(ZSA), we take a combined approach: we define an Orchard transaction as containing a
bundle of actions, where each action is both a spend and an output. This provides the same
inherent arity-hiding as multi-JoinSplit Sprout, but using Sapling value commitments to
balance the transaction without doubling its size.

## Dummy notes for Orchard

For Orchard, a transaction is a bundle of actions. Each action is composed of one spend and one output.
This means we have the same amount of "spends" and "outputs" in one transaction.
If we would like to create a transaction with a different number of spends and outputs,
we need to add "dummy" spends or outputs to balance their count.
A dummy spend or output is a note with a value of zero and a random recipient address.
In the ZK proof, when the value of the spent note is zero,
we do not verify that the corresponding spent note commitment is part of the Merkle tree.

## Split notes for OrchardZSA

For OrchardZSA, if the number of inputs exceeds the number of outputs,
we use dummy output notes (as in Orchard) to fill all actions.
Conversely, if the number of outputs exceeds the number of inputs, we use split notes to fill the actions.
In OrchardZSA, ensuring that the AssetBase is correctly created is crucial.
For this reason, split notes are used instead of dummy spent notes.
Split notes are essentially duplicates of actual spent notes,
but with the following differences:
- The nullifier is randomized to prevent it from being treated as double-spending.
- Its value is excluded from the transaction's or bundle's value balance.

Within the ZK proof, we verify that the commitment of each spent note (including split notes)
is part of the Merkle tree. This ensures that the AssetBase is constructed properly,
a note associated with this AssetBase exists within the Merkle tree.

For further details about split notes, refer to
[ZIP226](https://github.com/zcash/zips/blob/main/zips/zip-0226.rst).

## Memo fields

Each Orchard action has a memo field for its corresponding output, as with Sprout and
Sapling. We did at one point consider having a single Orchard memo field per transaction,
and/or having a mechanism for enabling multiple recipients to decrypt the same memo, but
these were decided against in order to keep the overall design simpler.


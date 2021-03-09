# Requirements

The design of Orchard aims to be mostly identical to the Sapling circuit in functionality.

## Functional Protocol Requirements

Functional protocol requirements aim to objectively define what the protocol achieves in a manner that can be verified by any reviewer with sufficient domain expertise.

Functional protocol requirements are scoped to the protocol design itself, which excludes important broader considerations for Orchard as a whole, such as usability goals for a wallet which aren't directly constrained by the protocol.

Finally, there is a category of proposed requirements that could have been coherently in-scope for Orchard which were excluded due to time & complexity constraints.

These scope clarifications exlude some draft requirements found in earlier revisions of this document, in pull request comments, or elsewhere. Many of those could become ZIPs in their own right.

### ProtoR1. Orchard Relies on a Zero-Knowledge Proving System that is Secure in The Uniform Random String Model

**Requirement:** The Orchard shielded transfer circuit uses a Zero-Knowledge Proving System that is secure in the Uniform Random String (URS) Model, and therefore has no reliance on a trusted setup.

**Rationale - UX:** Users who rely on Orchard are not directly vulnerable to the class of trusted parameter compromises.

**Rationale - Strategic:** As Sprout and Sapling usage declines or are deprecated, the ZEC a whole will become protected from any previous trusted setup compromise henceforth.

**Rationale - Strategic:** By avoiding a requirement on a trusted setup, Orchard *and future* upgrades no longer need a parameter setup MPC process, which improves protocol agility and the ability to respond quickly to security vulnerabilities.

### ProtoR2. Orchard Addresses

**Requirement:** Orchard introduces a new address format with an unambiguous user-facing encoding.

**Rationale - UX:** A new address format allows users to identify which ZKP platform their funds rely on.

**Rationale - UX:** A new address format reinforces the fact that users must migrate funds to opt into Orchard.

**Rationale - UX:** A new address format, in combination with the turnstile design, signals to users a boundary for turnstile-based supply integrity protections.

**Rationale - UX:** A new address format, in combination with the turnstile design, signals to users a boundary for privacy-guarantees.

**Counter-rationale - UX:** This complicates the simplified narrative of "z→z has strong privacy" because migrating between pools has a privacy cost.

**Rationale - Engineering:** A new address format simplifies cryptographic design and performance.

### ProtoR3. An Orchard Shielded Pool and Turnstile

**Requirement:** A new shielded pool is introduced with Orchard. ZEC may only enter the Orchard pool from t-addrs or shielded migration transactions, following the same "turnstile" design as for Sprout and Sapling.

**Rationale - UX:** A separate shielded pool clarifies which funds rely on which underlying ZKP/cryptographic platform.

**Rationale - UX:** A separate shielded pool associated with a new address type makes it easier to reason about migration.

### ProtoR4. Orchard Privacy Protections are Independent of Soundness.

**Requirement:** Orchard's privacy protections are independent of proving system soundness (in the sense that the security arguments are disentangled from each other and can be separately verified) so that a soundness compromise does not compromise any privacy protections.

**Rationale - UX:** Users can reasonably rely on the full privacy protections of Orchard even in the face of an Orchard soundness compromise, such as a successful counterfeiting attack.

## Non-functional Requirements

Non-functional requirements may not be verifiable from the protocol specification itself, but never-the-less define the goals, processes, or assessments used in delivering the Orchard protocol.

### Technical Strategy Requirements

These requirements define and rationalize some high-level goal which may not be apparent from the design specification.

#### TSR1. Sapling-equivalent Functionality

**Requirement:** Unless other product or engineering concerns dictate, the functionality of
Orchard should match that of the Sapling shielded protocol.

**Rationale - UX:** By being functional equivalent to Sapling, the existing Sapling
user base can immediately carry over most conceptual knowledge and UX habits from
Sapling to Orchard.

**Rationale - Technical Strategy:** Orchard is the first step in removing the trusted setup
attack surface as well as in deploying Halo 2. By targeting Sapling shielded protocol functionality,
the scope of Orchard R&D is greatly simplified to lower the strategic technical risk.

#### TSR2. Differences from Sapling Design Document

**Requirement:** Every observable difference from Sapling from the perspective of a Mobile Wallet SDK, a zcashd RPC developer, or any user of any known services built with those components, must be documented in a "Differences from Sapling" document in this repository. Any change that impacts a product-level requirement must exist in this document as an explicit requirement.

**Rationale - Technical Strategy:** Any deviation from Sapling needs to be clearly communicated to users and developers and reviewed by protocol engineers, product owners, and ecosystem stakeholders.

#### TSR3. Halo 2 Deployment

**Requirement:** Orchard must use Halo 2 for zero-knowledge proof validation.

**Rationale - Technical Strategy:** By selecting Halo 2 as the ZKP platform for Orchard, the Orchard deployment provides a forcing function to productionize Halo 2, thus unlocking potential future applications of recursion/proof-carrying data while limiting short-term risk.

#### TSR4. User Opt-in upon Activation

**Requirement:** Users may individually shield funds or migrate into a Orchard Pool as soon as the feature activates.

**Rationale - UX:** Users can act independently in opting into the Orchard Pool without coordinating globally.

#### TSR5. Full Design Verifiability

**Requirement:** Any arbitrary future reviewer should be able to verify the entire design and public state of the Zcash blockchain without relying on external authorities. Note that the Trusted Setup ceremonies for Sprout and Sapling violate this goal, and while Orchard activation itself does not achieve the goal, it can be achieved if Sprout and Sapling pools are eventually retired.

**Rationale - Technical Strategy:** A permissionless system requires the possibility of full-design validation by any arbitrary participant, and the need for this grows as the technology grows in importance.

#### TSR6. User Expectation about Pool Retirement

**Requirement:** Clarify in user-facing descriptions of Orchard the possibility and intent of eventual retirement of the pool.

**Rationale - UX:** While the Orchard specification does not itself define a pool retirement plan, retiring ZEC fund pools (and their underlying technology) generally is a desired goal in discussion across the Zcash development community. Users who are deciding whether or not to store funds in Orchard should be aware of this potential prior to making that decision. Note that the possibility of retirement is general to all pools, yet for existing pools that possibility is ambiguous. This requirement aims to avoid repeating that same expectation ambiguity starting with Orchard.

**Caveat:** This requirement does not stipulate the mechanism or time-table for retirement of Orchard funds. This is a current area of active discussion in the Zcash development community.

### Security & Safety Assessment Requirements

Requirments in this section clarify which areas of focus are used for external security assessments. These areas of focus are not required to be performed in separate distinct assessments, so for example a single security auditor may focus on multiple requirements in this section.

Furthermore, where "internal" assessments or evaluations are referenced or implied, these requirements were written with Electric Coin Company's internal security function in mind, although the general principles apply to any organization using this process.

#### SecR1. Halo 2 Security Proof

**Requirement:** A security proof of Halo 2 is published and receives sufficient review prior to protocol activation.

**Rationale - Security:** A security proof is one essential technique for security and safety assessment, and the proof must be public to meet the `TSR5` requirement.

#### SecR2. Expert Halo 2 Implementation Review

**Requirement:** One or more experts with domain expertise in Zero-knowledge proving system implementations has performed and published security reviews of the Halo 2 implementation prior to protocol activation.

**Rationale - Security:** An expert security review of the zero-knowledge proving implementation is essential, and as a newly emerging field this area deserves specialist scrutiny.

#### SecR3. Expert Zero-Knowledge Circuit Design & Implementation Review

**Requirement:** One or more experts with zero-knowledge circuit design domain knowledge has performed and published security reviews of the Orchard circuit design and implementation prior to protocol activation.

**Rationale - Security:** An expert security review of the circuit design and implementation is essential, and as a newly emerging field this area deserves specialist scrutiny.

#### SecR4. Expert General Cryptographic Security Design & Implementation Review

**Requirement:** One or more expert security reviews of general Orchard cryptographic design and implementation is performed and published prior to protocol activation.

**Rationale - Security:** Expert review by generalist cryptographic security reviewers helps discover issues in gaps that more specialized reviews may miss or not review.

#### SecR5. Expert General Information Security Design & Implementation Review

**Requirement:** One or more expert security reviews from information security generalists are performed and published prior to protocol activation.

**Rationale - Security:** Expert review by generalist security reviewers helps discover issues in gaps that more specialized reviews may miss or not review, especially covering potential gaps from specialist reviews around consensus, networking, memory safety, and a broad range of information security concerns.

#### SecR6. Security Assessment Re-evaluation Conditions

**Requirement:** If external security audits reveal many, or exceptionally concerning, issues, the safety of deploying Orchard will be re-evaluated internally with scrutiny.

**Rationale - Security:** When security assessment reveals issues which then get resolved, it can be tempting to assume this signifies all outstanding issues are fixed. However, when the nature or number of issues passes a reasonable threshold (based on security expert judgment) this should trigger a security re-evaluation across the board for Orchard.

### Non-Requirements

This section documents potential requirements that are explicitly not required, and their implementation is up to the product and engineering teams best judgement.

#### NonR1. Removing Sprout Functionality from the Consensus Protocol.

**Non-Requirement:** There is no requirement to change what is possible with Sprout funds at the consensus layer. (This leaves open the question of whether it will be possible to migrate funds directly from Sprout to Orchard within a single transaction.)

**Rationale - Engineering:** This reduces the scope of consensus changes associated with NU5. Similar effects can be obtained by non-consensus restrictions (or just omission of functionality) in wallets. Consensus-level restrictions would have required changes in the migration design of [ZIP 308](https://zips.z.cash/zip-0308). They may also have required changes to the JoinSplit circuit, which are infeasible in the proposed timeframe.

#### NonR2. User-Defined Asset Precursor Support

**Non-Requirement:** The protocol does not require precursor support for a future User-Defined Assets feature.

**Rationale - Technical Strategy:** Getting precursor support right requires certainty about a subset of UDA requirements, and blocking Orchard on clarifying future UDA requirements introduces more deployment & execution risk.

**Rationale - Engineering:** It appears to be possible to add UDAs in any case (modulo unresolved questions about economics and issuance policy) without explicit precursor support in this upgrade.

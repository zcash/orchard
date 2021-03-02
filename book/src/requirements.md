# Requirements

The design of Orchard aims to be mostly identical to the Sapling circuit in functionality.

## Scope of Requirements

These requirements are scoped to the protocol design itself, which excludes important broader considerations for Orchard as a whole, such as usability goals for a wallet which aren't directly constrained by the protocol. Additionally, an ideal criteria for these requirements is that any outside reviewer with sufficient domain expertise should be able to verify that the existing protocol specification meets the requirements in this document. This excludes requirements based on specific authorities / orgs / people.

These two scope clarifications exlude some draft requirements found in earlier revisions of this document, in pull request comments, or elsewhere. Many of those could become ZIPs in their own right.

## Protocol Product Requirements

### ProtoPR1. Orchard Relies on a Zeroknowledge Proving System which is Secure in The Uniform Random String Model

**Requirement:** The Orchard shielded transfer circuit uses a Zeroknowledge Proving System which is secure in the Uniform Random String (URS) Model, and therefore has no reliance on a trusted setup.

**Rationale UX:** Users who rely on Orchard are not directly vulnerable to the class of trusted parameter compromises.

**Rationale Strategic:** As Sprout and Sapling usage declines or are deprecated, the ZEC a whole will become protected from any previous trusted setup compromise henceforth.

### ProtoPR2. Orchard Addresses

**Requirement:** Orchard introduces a new address format with an unambiguous user-facing encoding.

**Rationale - UX:** A new address format allows users to identify which ZKP platform their funds rely on.

**Rationale - UX:** A new address format reinforces the fact that users must migrate funds to opt into Orchard.

**Rationale - UX:** A new address format, in combination with the turnstile design, signals to users a boundary for turnstile-based supply integrity protections.

**Rationale - UX:** A new address format, in combination with the turnstile design, signals to users a boundary for privacy-guarantees.

**Counter-rationale - UX:** This complicates the simplified narrative of "z→z has strong privacy" because migrating between pools has a privacy cost.

**Rationale - Engineering:** A new address format simplifies cryptographic design and performance.

### ProtoPR3. An Orchard Shielded Pool and Turnstile

**Requirement:** A new shielded pool is introduced with Orchard. ZEC may only enter the Orchard pool from t-addrs or shielded migration transactions, following the same "turnstile" design as for Sprout and Sapling.

**Rationale - UX:** A separate shielded pool clarifies which funds rely on which underlying ZKP/cryptographic platform.

**Rationale - UX:** A separate shielded pool associated with a new address type makes it easier to reason about migration.

## Technical Strategy Requirements

### TSR1. Sapling-equivalent Functionality

**Requirement:** Unless other product or engineering concerns dictate, the functionality of
Orchard should match that of the Sapling circuit.

**Rationale - UX:** By being functional equivalent to Sapling, the existing Sapling
userbase can immediately carry over most conceptual knowledge and UX habits from
Sapling to Orchard.

**Rationale - Technical Strategy:** Orchard is the first step in removing the toxic waste
vulnerability as well as in deploying Halo 2. By targeting Sapling circuit functionality,
the scope of Orchard R&D is greatly simplified to lower the strategic technical risk.

### TSR2. Differences from Sapling Design Document

**Requirement:** Every observable difference from Sapling from the perspective of a Mobile Wallet SDK, a zcashd RPC developer, or any user of any known services built with those components, must be documented in a "Differences from Sapling" document in this repository. Any change that impacts a product-level requirement must exist in this document as an explicit requirement.

**Rationale - Technical Strategy:** Any deviation from Sapling needs to be clearly communicated to users and developers and reviewed by protocol engineers, product owners, and ecosystem stakeholders.

### TSR3. Halo 2 Deployment

**Requirement:** Orchard must use Halo 2 for zero knowledge proof validation.

**Rationale - Technical Strategy:** By selecting Halo 2 as the ZKP platform for Orchard, the Orchard deployment provides a forcing function to productionize Halo 2.

### TSR4. User Opt-in upon Activation

**Requirement:** Users may individually shield funds or migrate into a Orchard Pool as soon as the feature activates.

**Rationale - UX:** Users can act independently in opting into the Orchard Pool without coordinating globally.

## Non-Requirements

This section documents potential requirements that are explicitly not required, and their implementation is up to the product and engineering teams best judgement.

### NonR2. User-Defined Asset Precursor Support

**Non-Requirement:** The protocol does not require precursor support for a future User-Defined Assets feature.

**Rationale - Technical Strategy:** Getting precursor support right requires certainty about a subset of UDA requirements, and blocking Orchard on clarifying future UDA requirements introduces more deployment & execution risk.

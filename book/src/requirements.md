# Requirements

The design of Orchard aimed to be mostly identical to the Sapling circuit in
functionality. We used this opportunity to clarify the requirements for Orchard at
the product and engineering levels.

## Protocol Product Requirements

### ProtoPR1. Orchard Addresses

**Requirement:** Orchard introduces a new address format with an unambiguous user-facing encoding.

**Rationale - UX:** A new address format allows users to identify which ZKP platform their funds rely on.

**Rationale - UX:** A new address format reinforces the fact that users must migrate funds to opt into Orchard.

**Rationale - UX:** A new address format, in combination with the turnstile design, signals to users a boundary for privacy-guarantees.

**Counter-rationale - UX:** This complicates the simplified narrative of "z→z has strong privacy" because migrating between pools has a privacy cost.

**Rationale - Engineering:** A new address format simplifies cryptographic design and performance.

### ProtoPR2. A Orchard Shielded Pool and Turnstile

**Requirement:** A new shielded pool is introduced with Orchard. ZEC may only enter the Orchard pool from t-addrs or shielded migration transactions, following the same "turnstile" design as for Sprout and Sapling.

**Rationale - UX:** A separate shielded pool clarifies which funds rely on which underlying ZKP/cryptographic platform.

**Rationale - UX:** A separate shielded pool associated with a new address type makes it easier to reason about migration.

**Rationale - UX:** FIXME - protects users funds from turnstile Sapling compromise, but cannot protect price, of course.

## Mobile Wallet SDK Product Requirements

### SDKPR1. Mobile Wallet SDK Orchard address generation from existing root secret.

**Requirement:** The ECC Mobile Wallet SDK supports generating new Orchard addresses based on a user's pre-existing backup/restore root secret prior to Mainnet Activation.

**Rationale - Engineering:** Wallet vendors who rely on the SDK can add Orchard features prior to Mainnet activation.

**Rationale - UX:** Wallet vendors can generate Orchard addresses without requiring users to back up any new secrets.

**Rationale - UX:** Wallet vendors can support importing compatible secrets in a new wallet which will have access to pre-Orchard-activation funds and also support Orchard addresses.

### SDKPR2. Mobile Wallet SDK supports sending to Orchard addresses.

**Requirement:** The ECC Mobile Wallet SDK supports sending funds to a Orchard address from a T-Address or a Orchard address upon Orchard activation.

**Rationale - Engineering:** Wallet vendors who rely on the SDK can add Orchard features prior to Mainnet activation.

### SDKPR3. Mobile Wallet SDK supports receiving to Orchard addresses.

**Requirement:** The ECC Mobile Wallet SDK supports receiving funds at a Orchard address from a T-Address or a Orchard address upon Orchard activation.

**Rationale - Engineering:** Wallet vendors who rely on the SDK can add Orchard features prior to Mainnet activation.

### SDKPR4. Mobile Wallet SDK supports turnstile migration from Sapling to Orchard addresses.

**Requirement:** The ECC Mobile Wallet SDK supports migrating funds from one or more Sapling addresses to a Orchard address.

**Rationale - Engineering:** Wallet vendors who rely on the SDK can add Orchard migration support prior to Mainnet activation.

### SDKPR5. Mobile Wallet SDK Orchard viewing key support.

**Requirement:**  ECC Mobile Wallet SDK supports exporting and importing Orchard viewing keys, and scanning private transaction data based on those viewing keys.

**Rationale - Engineering:** Wallet vendors who rely on the SDK can add viewing key features for Orchard.

## Mobile Wallet UX/UI Product Requirements

### UXPR1. Mobile Wallet UI supports generating at least one Orchard address *prior* to Mainnet activation.

**Requirement:** The ECC Mobile Wallet UI supports generating new Orchard addresses using the SDK *prior* to Mainnet activation. Prior to Orchard activation the UI indicates that the Orchard address cannot receive or send funds until Orchard activation.

**Rationale - UX:** Users can create and share their Orchard addresses with production products prior to activation.

### UXPR2. Mobile Wallet UX flow for sending transactions supports sending from or to Orchard addresses.

**Requirement:** The ECC Mobile Wallet UX flow supports sending to or from Orchard addresses where all source or destination addresses are either Orchard addresses or T-Addresses.

**Rationale - UX:** Users can use Orchard addresses for sending or receiving funds immediately upon Mainnet activation.

### UXPR3. Mobile Wallet guidance/educational UX for Sapling <-> Orchard transfer functionality.

**Requirement:** The ECC Mobile Wallet send-flow handles every case of mixing Sapling and Orchard addresses in user input, indicates clearly that this is not possible, provides (or links to) user education about this restriction, and provides instructions on how to enable Sapling -> Orchard migration.

**Rationale - UX:** It is very likely users will exercise these flows and this is *the* directly relevant hot-spot for user education and migration prompting.

### UXPR4. Mobile Wallet Sapling -> Orchard migration functionality.

**Requirement:** The ECC Mobile Wallet streamlines the ability to initiate and track the migration of funds from a Sapling address to a Orchard address. The wallet prioritizes the usage of Orchard addresses to nudge users to prefer that address and funds storage moving forward.

**Rationale - UX:** Migration must be simple in order to ensure users can quickly and easily begin using Orchard.

**Rationale - Strategic:** The longer the ecosystem straddles Sapling & Orchard the worse the impact on adoption will be, because of separate pools and no direct Sapling <-> Orchard payment support.

### UXPR5. Mobile Wallet Sapling -> Orchard continual migration functionality.

**Requirement:** The ECC Mobile Wallet must _always_ remember the Sapling addresses generated from the same backup secret as a Orchard address, and must automatically migrate any funds transferred to the Sapling address into the future. The UX must indicate when funds arrive at the Sapling address, the fact that they are automatically migrating, and prompt users to contact the senders to urge them to switch to the newer Orchard address.

**Rationale - UX:** Other parties may have bookmarked a user's Sapling address even after the user migrates to a Orchard address, and the wallet must ensure they do not lose funds.

**Rationale - UX:** Other parties may have bookmarked a user's Sapling address even after the user migrates to a Orchard address, and the wallet should help the user distribute their newer address to all of their financial peers.

**Rationale - Strategic:** We believe streamlining Orchard adoption by handling this edge case automatically lowers the a barrier to adoption by removing the need to coordinate a "switch-over date" with all counterparties.

### UXPR6. Mobile Wallet UX *OR* a new separate distinct mobile app supports importing and exporting any number of viewing keys, and viewing transaction history based on those viewing keys.

## lightwalletd Product Requirements

### LWDPR1. Full support of Light Wallet SDK & UX

**Requirement:** Any functionality required to support any of the Light Wallet SDK or Light Wallet UX requirements must be supported.

## Zcashd Full Node Wallet Product Requirements

### ZcashdPR1. Equivalent Support to Mobile SDK for Orchard.

**Requirement:** `zcashd` must support *every* Mobile Wallet SDK requirement.

**Rationale - UX:** Users who need any Orchard wallet functionality supported by ECC products must be able to use either the Mobile Wallet SDK or `zcashd` for any supported Orchard functionality. (In other words, they can choose the better option for their usage based on other factors rather than supported features.)

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

### NonR1. Wallet SDK and/or Zcashd RPC backwards compatibility.

**Non-Requirement:** In meeting the requirements for Orchard, the APIs of the Mobile Wallet SDK and Zcash RPC interface are not required to be backwards compatible. In other words, developers may be required to alter their existing API consumer code in order to support Orchard. This includes even the kind of RPC protocol for interaction with `zcashd`.

**Rationale - Engineering:** While backwards compatibility can help in lowering adoption costs, maintaining incongruent APIs for backwards compatibility can introduce maintenance, performance, and/or deployment costs both inside the API provider codebase (ex: `zcashd`) as well as in consumer codebases (ex: a relatively new wallet or service that doesn't benefit from handling older edge cases in `zcashd RPC`).

### NonR2. User-Defined Asset Precursor Support

**Non-Requirement:** The protocol does not require precursor support for a future User-Defined Assets feature.

**Rationale - Technical Strategy:** Getting precursor support right requires certainty about a subset of UDA requirements, and blocking Orchard on clarifying future UDA requirements introduces more deployment & execution risk.
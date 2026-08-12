# Licensing And Commercial Architecture

> Product and architecture planning only. Not legal advice.

## Current License Audit

| Area | Current repo evidence | Status |
|---|---|---|
| Root license files | `LICENSE-MIT`, `LICENSE-APACHE` | dual-license files present |
| README license wording | `README.md`, `README.zh-CN.md` say MIT/Apache-2.0, user's option | dual-license claim |
| Contribution license | `CONTRIBUTING.md` says contributions use the same dual license | dual-license contribution policy |
| Cargo package field | `aidisk/Cargo.toml` has `license = "MIT"` | inconsistent with README/contribution docs |
| Version | `aidisk/Cargo.toml` version `1.7.0` | unchanged by M0 |
| Third-party dependency licenses | `cargo tree --format "{p} {l}"` shows MIT, Apache-2.0, Unlicense, BSL-1.0, Unicode-3.0 combinations | needs full legal review before commercial release |

Product decision: M0 documents the inconsistency but does not change license files, Cargo metadata, contribution terms, or versioning.

## Open Source vs Source-Available

External-source fact: the Open Source Initiative states that open source licenses must allow free redistribution and must not discriminate against fields of endeavor, including business use. Source: `https://opensource.org/osd`.

Product decision: any future license that restricts commercial use, competitor use, or a field of endeavor should be described as source-available unless an OSI-approved license is chosen.

## Commercial Architecture Hypothesis

Hypothesis / to validate:

```text
Open Source Core
    +-- CLI / Agent interfaces
    +-- safety / scan / plan / quarantine / restore
    +-- structured asset intelligence

Commercial Desktop
    +-- premium human UX
    +-- continuous health
    +-- advanced Recovery Center
    +-- growth timeline
    +-- smart retention
    +-- scheduled governance UX
    +-- commercial packaging / entitlement
```

Exact boundaries require owner and legal review.

## License / Commercial Decision Table

| Option | Advantages | Risks / constraints | M0 recommendation |
|---|---|---|---|
| Keep MIT/Apache Core | maximum adoption, simplest integration, clean proprietary Desktop path | allows commercial forks/repackaging of Core | preferred near-term Core posture |
| Future GPLv3 Core | genuine open source with stronger copyleft | historical MIT/Apache releases remain; Desktop linkage boundaries need legal review | defer; do not change now |
| Source-available restrictions | stronger control over commercial reuse | not OSI Open Source if commercial/field restrictions apply; may reduce adoption | defer; only with explicit owner/legal decision |
| Separate proprietary Desktop | clean public/private boundary, simpler entitlement isolation | requires stable Core contract and release/version coordination | likely preferred commercial packaging path |

## Repository Boundary Options

| Model | Fit | Concerns | Recommendation |
|---|---|---|---|
| Public Core repo + private Desktop repo | clean OSS/proprietary separation | contract/version coordination required | preferred default if Desktop becomes commercial |
| Public monorepo with proprietary modules excluded/private | easier shared development | high risk of accidental proprietary leakage or confusing contribution boundary | not preferred |
| Public Core executable/API contract + private Desktop separate process | strong separation and language/tool flexibility | IPC/CLI contract must be stable and tested | strong fallback if license boundaries get complex |

## Legal Review Items

Qualified legal review is required before any commercial release or license transition for:

- resolving Cargo `license = "MIT"` vs repository dual-license language
- contributor IP policy, DCO vs CLA, and relicensing rights
- GPLv3 or source-available transition feasibility
- trademark/brand-use policy for AI Disk Doctor
- third-party dependency license compatibility and notice obligations
- proprietary Desktop distribution, signing, auto-update, and entitlement terms

## M0 Constraints

M0 must not:

- change license files
- create a custom license
- add CLA text
- implement billing, entitlement, auth, or license keys
- move Core code private
- make legal conclusions

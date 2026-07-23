# Citations

Master list of official source references used or referenced across this
documentation suite. Every substantive procedural/legal claim in
VoteAssist India's product content must trace back to a source of this
kind (or be explicitly labeled "community guidance — pending
verification," per `06-legal-compliance-review.md` and
`09-knowledge-base-schema.md`). This file is the human-readable index;
`citations` rows in `08-database-schema.md` are the storage-layer
projection, and each `knowledge_base_entries` row points to one or more of
these.

Entries marked "(verify exact URL/date)" indicate the source category and
general claim are correct per the brief this documentation suite was built
from, but the specific deep-link URL, circular number, or date must be
confirmed against the live site before being hardcoded into shipped
product copy — consistent with this project's rule of never asserting
unverified specifics.

## ECI Primary Portals

| Source | URL | Supports |
|---|---|---|
| Voters' Services Portal (unified) | https://voters.eci.gov.in | Primary deep-link target for Form 6/6A/7/8, e-EPIC download, roll search; superseded/merged the earlier NVSP (nvsp.in) |
| ECINET | https://ecinet.eci.gov.in | Deep-link target referenced alongside voters.eci.gov.in for elector services and "Book-a-Call with BLO" |
| Election Commission of India (main site) | https://eci.gov.in | General ECI information, circulars, notifications, press releases |
| ECI SVEEP (Systematic Voters' Education and Electoral Participation) | https://ecisveep.nic.in | Voter-education FAQs; source for student ordinary-residence guidance and general voter-awareness content |

## Legal / Regulatory Sources

| Source | Reference | Supports |
|---|---|---|
| Representation of the People Act, 1950 | Act of Parliament | Statutory basis for electoral roll preparation and elector qualification |
| Representation of the People Act, 1951 | Act of Parliament | Statutory basis for conduct of elections, postal ballot provisions |
| Registration of Electors Rules, 1960 | Rules under RP Act 1950 | Original rules framework for elector registration forms and procedures |
| Registration of Electors (Amendment) Rules, 2022 | Gazette notification, 17 June 2022 (in force 1 Aug 2022) (verify exact gazette notification number/citation) | Consolidation of forms: Form 6 (new registration), Form 6A (overseas electors), Form 7 (objection/deletion), Form 8 (shifting/correction/duplicate EPIC/PwD marking, absorbing the discontinued Form 8A and Form 001) |
| ECI circulars/manuals on elector registration (verify specific circular numbers per topic) | eci.gov.in circulars section | Operational detail supporting form-specific guidance beyond the bare rules text |

## PIB (Press Information Bureau) Releases

| Source | Topic | Supports |
|---|---|---|
| PIB release(s) on the four qualifying dates reform (verify exact release date/URL) | Qualifying dates (1 Jan, 1 Apr, 1 Jul, 1 Oct) | Explains the RP Act amendment enabling four annual qualifying dates for age-18 eligibility, replacing the single 1 January date |
| PIB release(s) on National Voter Helpline / NGSP 2.0 (verify exact release date/URL) | 1800-11-1950 helpline, National Grievance Service Portal 2.0, 48-hour resolution target | Supports helpline hours, grievance-tracking, and ERO/DEO/CEO resolution-timeline claims |
| PIB release(s) on postal ballot / home voting facility for 85+ and PwD electors (verify exact release date/URL) | Form 12D, 5-day submission window | Supports home-voting eligibility and timing claims |
| PIB release on e-EPIC launch (25 Jan 2021) (verify exact release date/URL) | e-EPIC digital voter ID | Supports e-EPIC launch date, OTP-verification download process, legal equivalence to physical card |

## Other Referenced (Non-ECI) Sources

| Source | Supports |
|---|---|
| Digital Personal Data Protection Act, 2023 (Government of India) | Data minimization, purpose limitation, consent, and erasure principles in `06-legal-compliance-review.md` and `08-database-schema.md` |
| Rights of Persons with Disabilities Act, 2016 (Government of India) | Domestic legal framework referenced alongside WCAG 2.2 in `06-legal-compliance-review.md` and `12-accessibility-spec.md` |
| WCAG 2.2 (W3C Web Content Accessibility Guidelines) | Technical accessibility baseline in `12-accessibility-spec.md` |
| Eighth Schedule to the Constitution of India | Source list of the 22 scheduled languages targeted in `11-multilingual-strategy.md` |

## Maintenance Note

Per the content review process in `06-legal-compliance-review.md`, every
row in this table (and every citation it backs in the knowledge base)
should be re-checked against the live source on the recommended cadence
(quarterly, and immediately after any relevant ECI circular or Gazette
notification), with `last_verified_date` updated accordingly at the KB
entry level. This file itself should be updated whenever a new source is
added to any KB entry or terminal outcome, so it remains a complete index
rather than drifting out of sync with the actual content.

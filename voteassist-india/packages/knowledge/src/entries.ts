import type { KnowledgeEntry } from "./types";

// Curated source-of-truth JSON files live in /knowledge-base/sources at the
// repo root (shared across the whole project, not just this package) so
// non-engineers can review and PR knowledge changes without touching code.
import form6 from "../../../knowledge-base/sources/form-6.json" with { type: "json" };
import form6a from "../../../knowledge-base/sources/form-6a.json" with { type: "json" };
import form7 from "../../../knowledge-base/sources/form-7.json" with { type: "json" };
import form8 from "../../../knowledge-base/sources/form-8.json" with { type: "json" };
import qualifyingDates from "../../../knowledge-base/sources/qualifying-dates.json" with { type: "json" };
import ordinaryResidenceStudent from "../../../knowledge-base/sources/ordinary-residence-student.json" with { type: "json" };
import pwdHomeVoting from "../../../knowledge-base/sources/pwd-home-voting.json" with { type: "json" };
import eEpic from "../../../knowledge-base/sources/e-epic.json" with { type: "json" };
import helplineGrievance from "../../../knowledge-base/sources/helpline-grievance.json" with { type: "json" };
import rollSearchPollingStation from "../../../knowledge-base/sources/roll-search-polling-station.json" with { type: "json" };
import serviceVoter from "../../../knowledge-base/sources/service-voter.json" with { type: "json" };

export const knowledgeEntries: KnowledgeEntry[] = [
  form6,
  form6a,
  form7,
  form8,
  qualifyingDates,
  ordinaryResidenceStudent,
  pwdHomeVoting,
  eEpic,
  helplineGrievance,
  rollSearchPollingStation,
  serviceVoter,
] as KnowledgeEntry[];

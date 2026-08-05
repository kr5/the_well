//! The four annual qualifying dates for age-18 elector eligibility, and the
//! "which one applies" computation.
//!
//! Until the Election Laws (Amendment) Act, 2021 (in force with the
//! Registration of Electors (Amendment) Rules, 2022), Indian electoral law
//! recognised a single qualifying date, 1 January, per Representation of
//! the People Act, 1950 s.14(b): to be enrolled, a person had to turn 18 on
//! or before 1 January of the year of that roll's revision — someone
//! turning 18 on, say, 15 March had to wait roughly nine months for the
//! *next* 1 January before they could even apply. The 2021/2022 amendment
//! (on ECI's own recommendation) added three more qualifying dates — 1
//! April, 1 July, 1 October — so a person can now become eligible up to
//! four times a year instead of once. See the `qualifying-dates`
//! knowledge-base entry for the citizen-facing explanation this module's
//! computation backs.
//!
//! What this module deliberately does NOT model: the separate,
//! per-notification "how far in advance of a qualifying date can I submit
//! Form 6" administrative window. That's an operational filing-window
//! detail set by ECI notification, not a standing legal constant — this
//! module only answers the substantive legal question ("are you 18 as on
//! the applicable qualifying date"), which doesn't need that window to be
//! correct: the computation below always resolves to the *soonest*
//! qualifying date on or after the date being assessed, and an applicant
//! who is 18 as on that soonest date is unconditionally eligible with
//! reference to it.

use chrono::{Datelike, NaiveDate};

/// The four qualifying dates for a given calendar year, in order.
pub fn qualifying_dates_for_year(year: i32) -> [NaiveDate; 4] {
    [
        ymd(year, 1, 1),
        ymd(year, 4, 1),
        ymd(year, 7, 1),
        ymd(year, 10, 1),
    ]
}

fn ymd(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_else(|| {
        panic!("1st of month {month} in year {year} must always be a valid date")
    })
}

/// The qualifying date an assessment made on `as_of` should be evaluated
/// against: the earliest of the four annual qualifying dates that falls on
/// or after `as_of` (rolling into 1 January of the following year if
/// `as_of` is after 1 October).
///
/// Why "on or after `as_of`", not "on or before": the age test itself
/// ("18 as on the qualifying date") is monotonic in time — anyone who
/// already satisfies it against a later qualifying date also satisfies it
/// against every qualifying date after that one. So testing against the
/// *soonest* upcoming qualifying date is simultaneously: the correct test
/// for someone applying in advance of turning 18 (are they 18 by the very
/// next opportunity), and a test any already-18 adult trivially passes
/// (they were 18 well before the next qualifying date too). There is no
/// case where using the soonest upcoming qualifying date, rather than a
/// later one, produces an incorrect eligibility answer.
///
/// If `as_of` is itself exactly a qualifying date, that same date is
/// returned (the test is inclusive, matching the statutory "on or before"
/// wording).
pub fn applicable_qualifying_date(as_of: NaiveDate) -> NaiveDate {
    let this_year = qualifying_dates_for_year(as_of.year());
    this_year
        .into_iter()
        .find(|d| *d >= as_of)
        .unwrap_or_else(|| qualifying_dates_for_year(as_of.year() + 1)[0])
}

/// The qualifying date immediately after `qualifying_date` — used to
/// explain, when someone narrowly misses one qualifying date, which one
/// they'd next become eligible with reference to (assuming no further
/// changes to the qualifying-date scheme itself).
pub fn next_qualifying_date_after(qualifying_date: NaiveDate) -> NaiveDate {
    applicable_qualifying_date(
        qualifying_date
            .succ_opt()
            .expect("NaiveDate::succ_opt only fails at chrono's date range limits"),
    )
}

/// Whether a person born on `date_of_birth` has reached age 18 as on `on`,
/// using ordinary calendar-birthday computation (turns 18 on the calendar
/// anniversary of their birth date). This is the plain, commonly-understood
/// convention — it deliberately does NOT apply the English common-law
/// "attains an age the day before the anniversary" doctrine sometimes cited
/// in age-computation disputes elsewhere; this crate has not found that
/// doctrine documented as ECI/RPA practice and would rather use the
/// convention every citizen and BLO already expects than quietly import a
/// legal technicality this crate cannot back with an Indian-electoral-law
/// citation.
pub fn is_18_or_older_on(date_of_birth: NaiveDate, on: NaiveDate) -> bool {
    let mut age = on.year() - date_of_birth.year();
    if (on.month(), on.day()) < (date_of_birth.month(), date_of_birth.day()) {
        age -= 1;
    }
    age >= 18
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifying_dates_for_year_are_the_four_notified_dates() {
        assert_eq!(
            qualifying_dates_for_year(2026),
            [
                ymd(2026, 1, 1),
                ymd(2026, 4, 1),
                ymd(2026, 7, 1),
                ymd(2026, 10, 1),
            ]
        );
    }

    #[test]
    fn applicable_qualifying_date_is_inclusive_of_as_of_itself() {
        assert_eq!(applicable_qualifying_date(ymd(2026, 1, 1)), ymd(2026, 1, 1));
        assert_eq!(applicable_qualifying_date(ymd(2026, 4, 1)), ymd(2026, 4, 1));
    }

    #[test]
    fn applicable_qualifying_date_rolls_to_next_date_within_year() {
        assert_eq!(applicable_qualifying_date(ymd(2026, 1, 2)), ymd(2026, 4, 1));
        assert_eq!(applicable_qualifying_date(ymd(2026, 4, 2)), ymd(2026, 7, 1));
        assert_eq!(applicable_qualifying_date(ymd(2026, 7, 2)), ymd(2026, 10, 1));
    }

    #[test]
    fn applicable_qualifying_date_rolls_over_into_next_year_after_october() {
        assert_eq!(applicable_qualifying_date(ymd(2026, 10, 2)), ymd(2027, 1, 1));
        assert_eq!(applicable_qualifying_date(ymd(2026, 12, 31)), ymd(2027, 1, 1));
    }

    #[test]
    fn next_qualifying_date_after_advances_exactly_one_step() {
        assert_eq!(next_qualifying_date_after(ymd(2026, 1, 1)), ymd(2026, 4, 1));
        assert_eq!(next_qualifying_date_after(ymd(2026, 10, 1)), ymd(2027, 1, 1));
    }

    #[test]
    fn turns_18_exactly_on_a_qualifying_date_is_18_on_that_date() {
        // Born 1 January 2008 -> turns 18 on 1 January 2026, one of the
        // four qualifying dates.
        assert!(is_18_or_older_on(ymd(2008, 1, 1), ymd(2026, 1, 1)));
    }

    #[test]
    fn turns_18_one_day_after_a_qualifying_date_is_not_yet_18_on_it() {
        // Born 2 January 2008 -> turns 18 on 2 January 2026, the day AFTER
        // the 1 January qualifying date.
        assert!(!is_18_or_older_on(ymd(2008, 1, 2), ymd(2026, 1, 1)));
        assert!(is_18_or_older_on(ymd(2008, 1, 2), ymd(2026, 4, 1)));
    }

    #[test]
    fn turns_18_one_day_before_a_qualifying_date_is_18_on_it() {
        // Born 31 December 2007 -> turns 18 on 31 December 2025, before
        // the 1 January 2026 qualifying date.
        assert!(is_18_or_older_on(ymd(2007, 12, 31), ymd(2026, 1, 1)));
    }

    #[test]
    fn boundary_around_1_april_qualifying_date() {
        assert!(is_18_or_older_on(ymd(2008, 4, 1), ymd(2026, 4, 1)));
        assert!(!is_18_or_older_on(ymd(2008, 4, 2), ymd(2026, 4, 1)));
        assert!(is_18_or_older_on(ymd(2008, 3, 31), ymd(2026, 4, 1)));
    }

    #[test]
    fn boundary_around_1_july_qualifying_date() {
        assert!(is_18_or_older_on(ymd(2008, 7, 1), ymd(2026, 7, 1)));
        assert!(!is_18_or_older_on(ymd(2008, 7, 2), ymd(2026, 7, 1)));
        assert!(is_18_or_older_on(ymd(2008, 6, 30), ymd(2026, 7, 1)));
    }

    #[test]
    fn boundary_around_1_october_qualifying_date() {
        assert!(is_18_or_older_on(ymd(2008, 10, 1), ymd(2026, 10, 1)));
        assert!(!is_18_or_older_on(ymd(2008, 10, 2), ymd(2026, 10, 1)));
        assert!(is_18_or_older_on(ymd(2008, 9, 30), ymd(2026, 10, 1)));
    }

    #[test]
    fn leap_year_birth_date_does_not_panic_and_resolves_reasonably() {
        // Born 29 Feb 2008 (a leap year). On 1 Jan 2026 (a non-leap year's
        // qualifying date, which is before their birthday-month anyway).
        let dob = ymd(2008, 2, 29);
        assert!(!is_18_or_older_on(dob, ymd(2026, 1, 1)));
        // By 1 April 2026 they've had their (Feb/Mar-anniversary) birthday
        // pass under ordinary calendar computation.
        assert!(is_18_or_older_on(dob, ymd(2026, 4, 1)));
    }

    #[test]
    fn an_adult_far_past_18_trivially_satisfies_every_qualifying_date() {
        let dob = ymd(1970, 6, 15);
        for as_of in [ymd(2026, 1, 1), ymd(2026, 4, 1), ymd(2026, 7, 1), ymd(2026, 10, 1)] {
            assert!(is_18_or_older_on(dob, as_of));
        }
    }
}

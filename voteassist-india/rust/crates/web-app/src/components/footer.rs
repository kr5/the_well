use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <nav aria-label="Footer">
                <A href="/about/what-this-is">"What this is"</A>
                <A href="/about/legal">"Terms"</A>
                <A href="/about/privacy">"Privacy"</A>
                <A href="/about/open-source">"Open source"</A>
                <A href="/feedback">"Feedback"</A>
                <A href="/accessibility">"Accessibility"</A>
            </nav>
            <p class="footer-disclaimer">
                "This platform provides general procedural guidance only. It is not legal advice, "
                "does not take political positions, and always defers to the Election Commission "
                "of India for anything official. For official action, use "
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer">"voters.eci.gov.in"</a>
                " or call the National Voter Helpline at 1800-11-1950."
            </p>
        </footer>
    }
}

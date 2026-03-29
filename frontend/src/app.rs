use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::nav_bar::NavBar;
use crate::pages::{
    home::Home,
    issue_detail::IssueDetail,
    issue_list::IssueList,
    login::Login,
    notifications::Notifications,
    pr_detail::PrDetail,
    pr_list::PrList,
    profile::Profile,
    repo_detail::RepoDetail,
    repo_list::RepoList,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="app-container">
                <Routes fallback=|| view! { <p>"Page not found"</p> }>
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/repos") view=RepoList />
                    <Route path=path!("/repos/:owner/:name") view=RepoDetail />
                    <Route path=path!("/repos/:owner/:name/issues") view=IssueList />
                    <Route path=path!("/repos/:owner/:name/issues/:index") view=IssueDetail />
                    <Route path=path!("/repos/:owner/:name/pulls") view=PrList />
                    <Route path=path!("/repos/:owner/:name/pulls/:index") view=PrDetail />
                    <Route path=path!("/notifications") view=Notifications />
                    <Route path=path!("/profile/:username") view=Profile />
                </Routes>
            </main>
            <NavBar />
        </Router>
    }
}

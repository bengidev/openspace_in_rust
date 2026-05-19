//! Active stage tracked by the onboarding router.
//!
//! Both variants are boxed so the enum itself stays small. Each
//! stage holds non-trivial state (the welcome state holds animation
//! timestamps and a persistence handle; the home state holds the
//! full app shell with router, theme, command registry, audit
//! sink), so boxing avoids moving large amounts of data on every
//! transition and keeps the router state itself cheap to swap.

use openspace_home::presenter::AppShell;
use openspace_welcome::WelcomeState;

pub enum Stage {
    Welcome(Box<WelcomeState>),
    Home(Box<AppShell>),
}

impl std::fmt::Debug for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Welcome(_) => f.debug_struct("Stage::Welcome").finish(),
            Stage::Home(_) => f.debug_struct("Stage::Home").finish(),
        }
    }
}

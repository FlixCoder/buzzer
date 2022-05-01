//! Frontend routes

use uuid::Uuid;
use yew::{function_component, html, use_reducer_eq, ContextProvider, Html, UseReducerHandle};
use yew_router::{BrowserRouter, Routable, Switch};

use crate::components::{GlobalState, Index, Room, UserEdit};

/// Frontend routes via the yew router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Routable)]
pub enum Routes {
	/// Index
	#[at("/")]
	Index,
	/// User info editor
	#[at("/user")]
	UserEdit,
	/// Buzzer Room
	#[at("/:id")]
	Room {
		/// Room ID
		id: Uuid,
	},
	/// Not Found Page
	#[not_found]
	#[at("/404")]
	NotFound,
}

/// Convert route to HTML output via yew components.
fn switch(routes: &Routes) -> Html {
	match routes {
		Routes::Index => html! { <Index /> },
		Routes::UserEdit => html! { <UserEdit /> },
		Routes::Room { id } => html! { <Room id={*id} /> },
		Routes::NotFound => html! { <h1>{ "404 - Not Found!" }</h1> },
	}
}

/// ContextProvider type for the global context
pub type GlobalContext = UseReducerHandle<GlobalState>;

/// Main app component is router to all the main app components.
#[function_component(AppRouter)]
pub fn app_router() -> Html {
	let state = use_reducer_eq(GlobalState::default);

	html! {
		<ContextProvider<GlobalContext> context={state}>
			<BrowserRouter>
				<Switch<Routes> render={ Switch::render(switch) } />
			</BrowserRouter>
		</ContextProvider<GlobalContext>>
	}
}

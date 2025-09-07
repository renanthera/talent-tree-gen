use leptos::{either::Either, prelude::*};
use serde::{Deserialize, Serialize};
use std::fmt;

use leptos::ev::{Event, Targeted};
use leptos::leptos_dom::logging::console_log;
use leptos::web_sys::HtmlSelectElement;

use crate::talent_encoding::TalentEncoding;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    WOW,
    #[allow(non_camel_case_types)]
    WOW_BETA,
    WOWDEV,
    WOWT,
    WOWXPTR,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub product: ProductType,
    pub major: usize,
    pub patch: usize,
    pub minor: usize,
    pub build: usize,
}

impl fmt::Display for ProductType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductType::WOW => write!(f, "Live"),
            ProductType::WOW_BETA => write!(f, "Beta"),
            ProductType::WOWDEV => write!(f, "Alpha"),
            ProductType::WOWT => write!(f, "PTR"),
            ProductType::WOWXPTR => write!(f, "XPTR"),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}.{}.{}-{}",
            self.product, self.major, self.patch, self.minor, self.build
        )
    }
}

async fn fetch_versions() -> Result<Vec<TalentEncoding>, Error> {
    Ok(reqwasm::http::Request::get("/versions.json")
        .send()
        .await?
        .json()
        .await?)
}

#[component]
pub fn VersionView() -> impl IntoView {
    let (selected_talent_encoding, set_selected_talent_encoding) =
        signal(TalentEncoding::default());

    let version_data = LocalResource::new(move || fetch_versions());

    let update_selection = move |tag_val: Targeted<Event, HtmlSelectElement>| {
        console_log(&tag_val.target().value());

        let find_encoding = |encoding: &TalentEncoding| -> bool {
            tag_val.target().value() == encoding.version.to_string()
        };

        match version_data.get() {
            Some(Ok(versions)) => set_selected_talent_encoding
                .set(versions.into_iter().find(find_encoding).unwrap_or_default()),
            _ => return, // handle None, Error
        }
    };

    let transition_fallback = || {
        view! { <option>"Loading..."</option> }
    };

    view! {
        <select name="version" on:input:target=update_selection>
            <option value="default">{move || format!("{0}", TalentEncoding::default())}</option>
            <Transition fallback=transition_fallback>
                {move || Suspend::new(async move {
                    version_data
                        .await
                        .map(|versions| {
                            view! {
                                <For
                                    each=move || versions.clone()
                                    key=|state| state.version.to_string()
                                    let(child)
                                >
                                    {match child {
                                        _ if child.version == Version::default() => {
                                            Either::Left(view! {})
                                        }
                                        _ => {
                                            let v = move || format!("{child}");
                                            Either::Right(
                                                view! { <option value=v.clone()>{v.clone()}</option> },
                                            )
                                        }
                                    }}
                                </For>
                            }
                        })
                })}
            </Transition>
        </select>
    }
}

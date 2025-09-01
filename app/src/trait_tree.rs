use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct TraitTree {
    trait_tree_id: i32,
    class_id: i32,
    spec_id: i32,
    class_name: String,
    spec_name: String,
}

async fn fetch_trait_trees() -> Result<Vec<TraitTree>, Error> {
    Ok(reqwasm::http::Request::get("/talent-data/talents.json")
        .send()
        .await?
        .json()
        .await?)
}

#[component]
pub fn TraitTreeDebug() -> impl IntoView {
    let ttdata = LocalResource::new(move || fetch_trait_trees());
    view! {
        <Transition fallback=|| view! { <div>"Loading..."</div> }>
            <ul>
                {move || Suspend::new(async move {
                    ttdata
                        .await
                        .map(|tt_data| {
                            tt_data
                                .into_iter()
                                .map(|tt| {
                                    // view! { <li>{format!("{tt:?}")}</li> }
                                    view! {
                                        <li>{format!("{0} - {1}", tt.class_name, tt.spec_name)}</li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                })}
            </ul>
        </Transition>
    }
}

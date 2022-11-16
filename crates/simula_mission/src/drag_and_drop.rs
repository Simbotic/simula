use crate::{
    account::Account,
    asset::Amount,
    asset_info::{AssetInfo, ImageTextureIds},
    utils::{trim_account, trim_wallet},
    wallet::Wallet,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

#[derive(Default)]
pub struct DragAndDropPlugin<T>
where
    T: AssetInfo,
{
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for DragAndDropPlugin<T>
where
    T: AssetInfo + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_system(drag_and_drop::<T>);
    }
}

pub fn drag_source(ui: &mut egui::Ui, id: egui::Id, body: impl FnOnce(&mut egui::Ui)) {
    let is_being_dragged = ui.memory().is_being_dragged(id);

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, egui::Sense::drag());
        if response.hovered() {
            ui.output().cursor_icon = egui::CursorIcon::Grab;
        }
    } else {
        ui.output().cursor_icon = egui::CursorIcon::Grabbing;

        // Paint the body to a new layer:
        let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
        let response = ui.with_layer_id(layer_id, body).response;

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);
        }
    }
}

pub fn drop_target<R>(
    ui: &mut egui::Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let is_being_dragged = ui.memory().is_anything_being_dragged();

    let margin = egui::Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect =
        egui::Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), egui::Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        // gray out:
        fill = egui::color::tint_color_towards(fill, ui.visuals().window_fill());
        stroke.color = egui::color::tint_color_towards(stroke.color, ui.visuals().window_fill());
    }

    ui.painter().set(
        where_to_put_background,
        egui::epaint::RectShape {
            rounding: style.rounding,
            fill,
            stroke,
            rect,
        },
    );

    egui::InnerResponse::new(ret, response)
}

pub fn drag_and_drop<T>(
    mut egui_ctx: ResMut<EguiContext>,
    wallets: Query<(&mut Wallet, &Children)>,
    accounts: Query<(&mut Account, &Children)>,
    mut assets: Query<(&mut T, &<T as AssetInfo>::AssetAttributes)>,
    mut commands: Commands,
    mut image_texture_ids: ResMut<ImageTextureIds>,
    mut asset_server: Res<AssetServer>,
) where
    T: AssetInfo,
{
    let mut ctx = egui_ctx.ctx_mut().clone();
    egui::Window::new("Transfer assets")
        .open(&mut true)
        .default_size(egui::vec2(256.0, 256.0))
        .vscroll(false)
        .resizable(true)
        .show(&mut ctx, |ui| {
            let id_source = "my_drag_and_drop_demo";
            let mut source_asset = None; //this will hold the dragged asset position
            let mut drop_account = None; //this holds the wallet and account where the asset is dropped

            ui.columns(wallets.into_iter().len(), |uis| {
                for (wallet_idx, wallet) in wallets.into_iter().enumerate() {
                    // iterate wallets

                    let ui = &mut uis[wallet_idx]; // our current column, index comes from the iteration of wallets

                    ui.add(egui::Label::new(format!(
                        "Wallet: {}",
                        trim_wallet(wallet.0)
                    )));

                    let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯

                    ui.set_min_size(egui::vec2(64.0, 100.0)); // set window size (To be Modified)

                    for (account_idx, account) in wallet.1.into_iter().enumerate() {
                        // iterate accounts

                        let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                            // Call the drag and drop function

                            if let Ok((account, account_assets)) = accounts.get(*account) {
                                // obtain al the assets from the current account

                                ui.add(egui::Label::new(trim_account(account)));

                                for (asset_idx, asset_entity) in account_assets.iter().enumerate() {
                                    // iterate assets
                                    if let Ok((asset, attributes)) = assets.get(*asset_entity) {
                                        let item_id = egui::Id::new(id_source)
                                            .with(wallet_idx)
                                            .with(account_idx)
                                            .with(asset_idx); // we create an id with all index

                                        if asset.is_draggable() {
                                            drag_source(ui, item_id, |ui| {
                                                //we make the asset dragable
                                                asset.render(
                                                    ui,
                                                    &mut image_texture_ids,
                                                    &mut asset_server,
                                                    &mut egui_ctx,
                                                    &attributes,
                                                );
                                            });
                                        } else {
                                            asset.render(
                                                ui,
                                                &mut image_texture_ids,
                                                &mut asset_server,
                                                &mut egui_ctx,
                                                &attributes,
                                            );
                                        }

                                        if ui.memory().is_being_dragged(item_id) {
                                            source_asset = Some(asset_entity); // we now know which asset is being dragged
                                        }
                                    }
                                }
                            }
                        })
                        .response;

                        let is_being_dragged = ui.memory().is_anything_being_dragged();

                        if is_being_dragged
                            && can_accept_what_is_being_dragged
                            && response.hovered()
                        {
                            drop_account = Some(account); //we store the drop target
                        }
                    }
                }
            });

            if let Some(source_asset) = source_asset {
                if let Some(drop_account) = drop_account {
                    if ui.input().pointer.any_released() {
                        let mut asset_tuple: (u64, u64, Amount) = (0, 0, 0.into());

                        if let Ok((asset, _attributes)) = assets.get(*source_asset) {
                            // save the amount, class and asset id to compare with the assets in dropped account
                            asset_tuple = (asset.class_id(), asset.asset_id(), asset.amount());
                        }

                        if let Ok(account) = accounts.get(*drop_account) {
                            // we check if it exists to add the amounts
                            let mut asset_exists = false;
                            for asset in account.1.iter() {
                                if let Ok((mut mut_asset, _attributes)) = assets.get_mut(*asset) {
                                    if mut_asset.drop(asset_tuple.0, asset_tuple.1, asset_tuple.2) {
                                        asset_exists = true;
                                    }
                                }
                            }
                            if !asset_exists {
                                //if the asset doesn't exist we push it to the dropped account
                                if let Ok((new_asset, _attributes)) = assets.get_mut(*source_asset)
                                {
                                    new_asset.push_as_children(&mut commands, *drop_account);
                                }
                            }
                        }
                        if let Ok((mut asset, _attributes)) = assets.get_mut(*source_asset) {
                            // finally we deplete the amount of the dragged assetREa
                            asset.drag();
                        }
                    }
                }
            }
        });
}

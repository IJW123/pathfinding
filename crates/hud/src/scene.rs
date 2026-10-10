//! The HUD's entity layout, authored as a BSN scene list: a text readout pinned top-right and a
//! scale-bar column pinned bottom-left. Pure layout — what each element *shows* each frame lives in
//! [`crate::systems`], keyed by the marker components set here.

use bevy::prelude::*;
use bevy::scene::{SceneList, bsn_list};

use crate::components::{HudReadout, ScaleBarFill, ScaleBarLabel};
use crate::constants::{
    HUD_FONT_SIZE, HUD_MARGIN_PX, HUD_TEXT_COLOR, SCALE_BAR_COLOR, SCALE_BAR_HEIGHT_PX,
    SCALE_BAR_ROW_GAP_PX, TARGET_BAR_PX,
};

/// Two root UI entities: the readout and the scale-bar column (label over fill bar). Spawn once at
/// startup via [`SpawnListSystem::spawn`](bevy::scene::SpawnListSystem::spawn). Font size is pinned
/// in px on purpose: bevy's default is `rem(1)`, which would rescale with the UI root.
#[must_use]
pub fn hud() -> impl SceneList {
    bsn_list! {
        Text("x: 0 m  y: 0 m\nz: 0.0 m\nspeed: 0.0 m/s")
        TextFont { font_size: FontSize::Px(HUD_FONT_SIZE) }
        TextColor(HUD_TEXT_COLOR)
        Node {
            position_type: PositionType::Absolute,
            top: px(HUD_MARGIN_PX),
            right: px(HUD_MARGIN_PX),
        }
        HudReadout
        --
        Node {
            position_type: PositionType::Absolute,
            bottom: px(HUD_MARGIN_PX),
            left: px(HUD_MARGIN_PX),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: px(SCALE_BAR_ROW_GAP_PX),
        }
        Children [
            Text("0 m")
            TextFont { font_size: FontSize::Px(HUD_FONT_SIZE) }
            TextColor(HUD_TEXT_COLOR)
            ScaleBarLabel
            --
            Node {
                width: px(TARGET_BAR_PX),
                height: px(SCALE_BAR_HEIGHT_PX),
            }
            BackgroundColor(SCALE_BAR_COLOR)
            ScaleBarFill
        ]
    }
}

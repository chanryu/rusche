//! Paint a Rusche scene list: `(clear …)`, `(rect …)`, `(sprite …)`.

use egui::{pos2, Color32, Pos2, Rect, Sense, Ui, Vec2};
use rusche::Expr;

use crate::sprites::{self, SpriteAtlas, SCENE_H, SCENE_W, SCALE};

#[derive(Debug)]
pub struct SceneError(pub String);

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn expect_sym<'a>(cmd: &'a str, expr: &'a Expr) -> Result<&'a str, SceneError> {
    expr.as_sym()
        .map(|s| s.as_str())
        .ok_or_else(|| SceneError(format!("{cmd}: expected a symbol, got {expr}")))
}

fn expect_num(cmd: &str, expr: &Expr) -> Result<f32, SceneError> {
    expr.as_num()
        .map(|n| n as f32)
        .ok_or_else(|| SceneError(format!("{cmd}: expected a number, got {expr}")))
}

fn rgb(r: f32, g: f32, b: f32) -> Color32 {
    Color32::from_rgb(
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

/// Draw `scene` into `ui` using `atlas` for named sprites.
pub fn paint(ui: &mut Ui, atlas: &SpriteAtlas, scene: &Expr) -> Result<Pos2, SceneError> {
    let list = scene
        .as_list()
        .ok_or_else(|| SceneError(format!("scene must be a list, got {scene}")))?;

    let desired = Vec2::new(SCENE_W * SCALE, SCENE_H * SCALE);
    let (response, painter) = ui.allocate_painter(desired, Sense::click_and_drag());
    let origin = response.rect.min;

    painter.rect_filled(response.rect, 0.0, Color32::from_rgb(247, 247, 247));

    for cmd_expr in list.iter() {
        let cmd_list = cmd_expr
            .as_list()
            .ok_or_else(|| SceneError(format!("command must be a list, got {cmd_expr}")))?;
        let items: Vec<&Expr> = cmd_list.iter().collect();
        if items.is_empty() {
            return Err(SceneError("empty command".into()));
        }
        let op = expect_sym("command", items[0])?;
        match op {
            "clear" => {
                if items.len() != 4 {
                    return Err(SceneError(format!(
                        "clear: expected 3 numbers, got {}",
                        items.len() - 1
                    )));
                }
                let bg = rgb(
                    expect_num("clear", items[1])?,
                    expect_num("clear", items[2])?,
                    expect_num("clear", items[3])?,
                );
                painter.rect_filled(response.rect, 0.0, bg);
            }
            "rect" => {
                if items.len() != 8 {
                    return Err(SceneError(format!(
                        "rect: expected x y w h r g b, got {} args",
                        items.len() - 1
                    )));
                }
                let x = expect_num("rect", items[1])?;
                let y = expect_num("rect", items[2])?;
                let w = expect_num("rect", items[3])?;
                let h = expect_num("rect", items[4])?;
                let color = rgb(
                    expect_num("rect", items[5])?,
                    expect_num("rect", items[6])?,
                    expect_num("rect", items[7])?,
                );
                let rect = Rect::from_min_size(
                    Pos2::new(origin.x + x * SCALE, origin.y + y * SCALE),
                    Vec2::new(w * SCALE, h * SCALE),
                );
                painter.rect_filled(rect, 0.0, color);
            }
            "sprite" => {
                if items.len() != 7 {
                    return Err(SceneError(format!(
                        "sprite: expected name x y r g b, got {} args",
                        items.len() - 1
                    )));
                }
                let name = expect_sym("sprite", items[1])?;
                let x = expect_num("sprite", items[2])?;
                let y = expect_num("sprite", items[3])?;
                let tint = sprites::tint(
                    expect_num("sprite", items[4])?,
                    expect_num("sprite", items[5])?,
                    expect_num("sprite", items[6])?,
                );
                let (tex, size) = atlas
                    .get(name)
                    .ok_or_else(|| SceneError(format!("unknown sprite `{name}`")))?;
                let rect = Rect::from_min_size(
                    Pos2::new(origin.x + x * SCALE, origin.y + y * SCALE),
                    size * SCALE,
                );
                painter.image(
                    tex.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    tint,
                );
            }
            other => return Err(SceneError(format!("unknown command `{other}`"))),
        }
    }

    Ok(origin)
}

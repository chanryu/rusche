//! Chrome-dino-style runner host: Rusche scripts own the game, egui paints the scene.

pub mod app;
pub mod scene;
pub mod sprites;

use rusche::{Error, ErrorKind, EvalError, Evaluator, Expr};

/// Bundled game scripts, evaluated in this order.
pub const SCRIPTS: &[(&str, &str)] = &[
    ("state.rsch", include_str!("rsch/state.rsch")),
    ("player.rsch", include_str!("rsch/player.rsch")),
    ("field.rsch", include_str!("rsch/field.rsch")),
    ("scene.rsch", include_str!("rsch/scene.rsch")),
    ("game.rsch", include_str!("rsch/game.rsch")),
];

/// Load the game scripts into a fresh evaluator with the prelude.
pub fn load_game() -> Result<(Evaluator, Expr), Error> {
    let evaluator = Evaluator::with_prelude();
    for (name, src) in SCRIPTS {
        evaluator.eval_str_named(src, name)?;
    }
    let tick = evaluator.root_env().lookup("tick").ok_or_else(|| {
        Error::from(EvalError::new(ErrorKind::Other, "tick is not defined"))
    })?;
    Ok((evaluator, tick))
}

/// Y position of the first matching sprite command, if any.
pub fn sprite_y(scene: &Expr, name: &str) -> Option<f64> {
    let list = scene.as_list()?;
    for cmd in list.iter() {
        let items: Vec<&Expr> = cmd.as_list()?.iter().collect();
        if items.len() >= 4
            && items[0].as_sym().map(|s| s.as_str()) == Some("sprite")
            && items[1].as_sym().map(|s| s.as_str()) == Some(name)
        {
            return items[3].as_num();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::Expr;

    #[test]
    fn jump_raises_dino() {
        let (evaluator, tick) = load_game().expect("load game");

        let grounded = evaluator
            .call(
                &tick,
                &[Expr::from(1.0 / 60.0), Expr::from(false), Expr::from(false)],
            )
            .expect("idle tick");
        let run_y = sprite_y(&grounded, "dino-run-0")
            .or_else(|| sprite_y(&grounded, "dino-run-1"))
            .expect("grounded dino sprite");

        let (evaluator, tick) = load_game().expect("reload");
        // Press jump on the first frame after load (prev-jump? is false).
        for _ in 0..3 {
            evaluator
                .call(
                    &tick,
                    &[Expr::from(1.0 / 60.0), Expr::from(true), Expr::from(false)],
                )
                .expect("jump tick");
        }
        let airborne = evaluator
            .call(
                &tick,
                &[Expr::from(1.0 / 60.0), Expr::from(true), Expr::from(false)],
            )
            .expect("jump tick");

        let jump_y = sprite_y(&airborne, "dino-jump").expect("jump sprite");
        assert!(
            jump_y < run_y,
            "expected jump y {jump_y} above grounded y {run_y}"
        );
    }

    #[test]
    fn runs_without_eval_error() {
        let (evaluator, tick) = load_game().expect("load game");
        let dt = Expr::from(1.0 / 60.0);
        // Start with a jump, then keep running; auto-jump when near the ground.
        evaluator
            .call(&tick, &[dt.clone(), Expr::from(true), Expr::from(false)])
            .expect("start");
        for i in 0..600 {
            let jump = i % 45 == 0;
            let scene = evaluator
                .call(&tick, &[dt.clone(), Expr::from(jump), Expr::from(false)])
                .unwrap_or_else(|e| panic!("tick {i}: {e}"));
            assert!(scene.as_list().is_some(), "scene must be a list");
        }
    }

    fn scene_has_sprite(scene: &Expr, name: &str) -> bool {
        scene.as_list().is_some_and(|list| {
            list.iter().any(|cmd| {
                cmd.as_list().is_some_and(|items| {
                    let items: Vec<&Expr> = items.iter().collect();
                    items.len() >= 2
                        && items[0].as_sym().map(|s| s.as_str()) == Some("sprite")
                        && items[1].as_sym().map(|s| s.as_str()) == Some(name)
                })
            })
        })
    }

    #[test]
    fn spawns_cactus_clusters() {
        let (evaluator, tick) = load_game().expect("load game");
        let dt = Expr::from(1.0 / 60.0);
        evaluator
            .call(&tick, &[dt.clone(), Expr::from(true), Expr::from(false)])
            .expect("start");
        let mut saw_cluster = false;
        for i in 0..900 {
            let jump = i % 45 == 0;
            let scene = evaluator
                .call(&tick, &[dt.clone(), Expr::from(jump), Expr::from(false)])
                .unwrap_or_else(|e| panic!("tick {i}: {e}"));
            if scene_has_sprite(&scene, "cactus-s2")
                || scene_has_sprite(&scene, "cactus-s3")
                || scene_has_sprite(&scene, "cactus-l2")
                || scene_has_sprite(&scene, "cactus-l3")
            {
                saw_cluster = true;
                break;
            }
        }
        assert!(saw_cluster, "expected a 2+ cactus cluster within ~15s");
    }
}

use std::{io, thread};
use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{ExecutableCommand, terminal};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use rusty_audio::Audio;

use invaders::frame::{self, Drawable, new_frame};
use invaders::invaders::Invaders;
use invaders::player::Player;
use invaders::render::{self};

fn main() -> Result<(), Box<dyn Error>> {
	let mut audio = Audio::new();
	audio.add("explosion1", "assets/sounds/zapsplat_explosion_grenade_300m_distance_reverb_ext_25222.mp3");
	audio.add("explosion2", "assets/sounds/audio_hero_Explosion7LargeBoo_TE026301_348.mp3");
	audio.add("gameover", "assets/sounds/zapsplat_multimedia_game_negative_lose_life_tone_17878.mp3");
	audio.add("move", "assets/sounds/zapsplat_science_fiction_robot_chirp_beep_metallic_short_004_55841.mp3");
	audio.add("shoot", "assets/sounds/pm_sfg_vol1_weapon_47_3_gun_gunshot_futuristic_365.mp3");
	audio.add("startup", "assets/sounds/jessey_drake_synth_space_weird_synthy_sci_fi_power_up_digital_glitches_2_snth_jd.mp3");
	audio.add("victory", "assets/sounds/zapsplat_multimedia_game_tone_short_positive_bonus_win_glide_synth_001_51329.mp3");

	audio.play("startup");


	let mut stdout = io::stdout();
	terminal::enable_raw_mode()?;
	stdout.execute(EnterAlternateScreen)?;
	stdout.execute(Hide)?;


	let (render_tx, render_rx) = mpsc::channel();
	let render_handle = thread::spawn(move || {
		let mut last_frame = frame::new_frame();
		let mut stdout = io::stdout();
		render::render(&mut stdout, &last_frame, &last_frame, true);
		loop {
			let curr_frame = match render_rx.recv() {
				Ok(f) => f,
				Err(_) => break,
			};

			render::render(&mut stdout, &last_frame, &curr_frame, false);
			last_frame = curr_frame;
		}
	});

	let mut player = Player::new();
	let mut instant = Instant::now();
	let mut invaders = Invaders::new();

	'gameloop: loop {
		let delta = instant.elapsed();
		instant = Instant::now();
		let mut curr_frame = new_frame();

		while event::poll(Duration::default())? {
			if let Event::Key(key_event) = event::read()? {
				match key_event.code {
					KeyCode::Left => player.move_left(),
					KeyCode::Right => player.move_right(),
					KeyCode::Char(' ') | KeyCode::Enter => {
						if player.shoot() {
							audio.play("shoot");
						}
					}
					KeyCode::Esc | KeyCode::Char('q') => {
						audio.play("gameover");
						break 'gameloop;
					}
					_ => {}
				}
			}
		}

		player.update(delta);
		if invaders.update(delta) {
			audio.play("move");
		}
		if player.detect_hits(&mut invaders) {
			if rand::random() {
				audio.play("explosion1");
			} else {
				audio.play("explosion2");
			}
		}

		let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
		for drawable in drawables {
			drawable.draw(&mut curr_frame);
		}
		let _ = render_tx.send(curr_frame);
		thread::sleep(Duration::from_millis(10));

		if invaders.destroyed() {
			audio.play("victory");
			break 'gameloop;
		} else if invaders.reached_bottom() {
			audio.play("gameover");
			break 'gameloop;
		}
	}

	// Shutdown process
	drop(render_tx);
	render_handle.join().unwrap();
	audio.wait();
	stdout.execute(Show)?;
	stdout.execute(LeaveAlternateScreen)?;
	let _ = terminal::disable_raw_mode();
	Ok(())
}

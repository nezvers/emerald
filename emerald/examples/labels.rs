use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(GamepadExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct ElapsedTime(f32);

pub struct GamepadExample {
    world: World,
}
impl Game for GamepadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let font = emd.loader().font("Roboto-Light.ttf", 40).unwrap();

        let mut left_aligned_label = Label::new("Emerald Engine", font, 80);
        left_aligned_label.max_width = Some(400.0);

        let mut centered_label = left_aligned_label.clone();
        centered_label.horizontal_align = HorizontalAlign::Center;

        let mut right_label = left_aligned_label.clone();
        right_label.horizontal_align = HorizontalAlign::Right;

        self.world
            .spawn((ElapsedTime(0.0), Transform::default(), left_aligned_label));
        self.world.spawn((
            ElapsedTime(0.0),
            Transform::from_translation((0.0, 300.0)),
            centered_label,
        ));
        self.world.spawn((
            ElapsedTime(0.0),
            Transform::from_translation((0.0, -300.0)),
            right_label,
        ));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        for (_, (label, _elapsed_time)) in
            self.world.query::<(&mut Label, &mut ElapsedTime)>().iter()
        {
            if input.is_key_just_pressed(KeyCode::A) {
                label.scale *= 0.5;
            } else if input.is_key_just_pressed(KeyCode::D) {
                label.scale *= 2.0;
            } else if input.is_key_just_pressed(KeyCode::E) {
                label.max_width = Some(800.0);
            } else if input.is_key_just_pressed(KeyCode::R) {
                label.max_width = Some(400.0);
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}

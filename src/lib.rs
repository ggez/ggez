

pub enum GameError {
    Lolwtf
}

pub trait State {
    fn init(&self) -> Result<(), GameError>;
    fn update(&self) -> Result<(), GameError>;
    fn draw(&self) -> Result<(), GameError>;
}

pub struct Engine<'e>
{
    states: Vec<&'e mut State>
}

impl<'e> Engine<'e> {
    pub fn new() -> Engine<'e>
    {
        Engine
        {
            states: Vec::new()
        }
    }

    pub fn add_obj(&mut self, s: &'e mut State)
    {
        self.states.push(s)
    }

    pub fn ralf(&mut self)
    {
        for s in &mut self.states
        {
            s.init();
        }
        loop
        {
            for s in &mut self.states
            {
                s.update();
            }
            for s in &mut self.states
            {
                s.draw();
            }
        }
    }
}

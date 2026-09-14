pub mod render{
   // Use the ratatui-sci-fi widget lib for faster widgets

    
    use std::{error::Error, io::Stdout, ops::{Add, Sub}, sync::{Arc, Mutex}};

use crossterm::{event::{DisableMouseCapture, KeyCode, KeyEvent}, execute, terminal::{LeaveAlternateScreen, disable_raw_mode}};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::Style};
use cpal::{Host, HostId};
use crossterm::event::{self, Event};
use ratatui::{widgets::{Block, Borders, List, ListItem}};

use cpal::traits::HostTrait; 
use std::time::Duration;
use ratatui::{widgets::{Paragraph, Wrap}, style::Color, text::{Line, Span}};

use crate::models::models::App;


   pub struct SelectionState {
      pub items: Vec<String>,
      pub selected: usize,
   }

   impl SelectionState {
      pub fn new(items: Vec<String>) -> Self {
         Self { items, selected: 0 }
      }

      pub fn next(&mut self) {
         if !self.items.is_empty() {
               self.selected = (self.selected + 1) % self.items.len();
         }
      }

      pub fn prev(&mut self) {
         if !self.items.is_empty() {
               self.selected = if self.selected == 0 {
                  self.items.len() - 1
               } else {
                  self.selected - 1
               };
         }
      }

      pub fn get_selected(&self) -> Option<&String> {
         self.items.get(self.selected)
      }
   }

   pub fn select_host(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<HostId, Box<dyn Error>> {        
      let hosts  = cpal::available_hosts().into_iter().collect::<Vec<HostId>>();
      let mut state = SelectionState::new(hosts.iter().map(|h| h.name().to_string()).collect::<Vec<String>>());
      terminal.clear()?;

      loop {
         terminal.draw(|f| {
               let size = f.area();
               let block = Block::default().title("Select audio host").borders(Borders::ALL);
               
               let list_items: Vec<ListItem> = state.items.iter().enumerate().map(|(i, name)| {
                  if i == state.selected {
                     ListItem::new(format!("> {}", name))
                  } else {
                     ListItem::new(format!("  {}", name))
                  }
               }).collect();
               let list = List::new(list_items).block(block).style(Style::default());
               f.render_widget(list, size);
         })?;

         if crossterm::event::poll(std::time::Duration::from_millis(100))? {
               if let Event::Key(key) = event::read()? {
                  match key.code {
                     KeyCode::Up | KeyCode::Tab => state.prev(),
                     KeyCode::Down | KeyCode::BackTab => state.next(),
                     KeyCode::Enter => {
                           if let Some(&host_id) = hosts.get(state.selected) {
                              terminal.clear()?;
                              return Ok(host_id);
                           }
                     },
                     KeyCode::Char('q') => {
                           disable_raw_mode()?;
                           execute!(&mut terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                           terminal.show_cursor()?;
                           std::process::exit(0);
                     }
                     _ => {}
                  }
               }
         }
      }
   }

   pub fn select_device(terminal: &mut Terminal<CrosstermBackend<Stdout>>, host: &Host) -> Result<String, Box<dyn Error>> {
      let mut state = SelectionState::new(host.input_devices()?.map(|d| d.to_string()).collect::<Vec<String>>());
      terminal.clear()?;

      loop {
         terminal.draw(|f| {
               let size = f.area();
               let block = Block::default().title("Select input device").borders(Borders::ALL);  
               let list_items: Vec<ListItem> = state.items.iter().enumerate().map(|(i, name)| {
                  if i == state.selected {
                     ListItem::new(format!("> {}", name))
                  } else {
                     ListItem::new(format!("  {}", name))
                  }
               }).collect();

               let list = ratatui::widgets::List::new(list_items).block(block).style(ratatui::style::Style::default());
               f.render_widget(list, size);
         })?;

         if crossterm::event::poll(std::time::Duration::from_millis(100))? {
               if let Event::Key(key) = event::read()? {
                  match key.code {
                     KeyCode::Up | KeyCode::BackTab => state.prev(),
                     KeyCode::Down | KeyCode::Tab => state.next(),
                     KeyCode::Enter => {
                           if let Some(device_name) = state.get_selected() {
                              terminal.clear()?;
                              return Ok(device_name.clone());
                           }
                     },
                     KeyCode::Char('q') => {
                           disable_raw_mode()?;
                           execute!(&mut terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                           terminal.show_cursor()?;
                           std::process::exit(0);
                     }
                     _ => {}
                  }
               }
         }
      }
   }

   pub fn run_app(terminal:&mut Terminal<CrosstermBackend<Stdout>>,app: Arc<Mutex<App>>) -> Result<(),Box<dyn Error>>{
      let mut last_tick = std::time::Instant::now();
      let tick_rate = Duration::from_millis(8); // 120 FPS

      loop {
         terminal.draw(|f| {
               let app_guard = app.lock().unwrap();
               // DRAW HERE
         })?;
         let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0));

         if crossterm::event::poll(timeout)? {
               if let Event::Key(key) = event::read()? {
                  let mut app_guard = app.lock().unwrap();
                  handle_input(&mut app_guard, key);
                  if key.code == KeyCode::Char('q') {
                     break;
                  }
               }
         }

         if last_tick.elapsed() >= tick_rate {
               last_tick = std::time::Instant::now();
         }
      }

      disable_raw_mode()?;
      execute!(&mut terminal.backend_mut(),LeaveAlternateScreen,DisableMouseCapture)?;
      terminal.show_cursor()?;
      Ok(())
   }

   fn handle_input(app: &mut App,key: KeyEvent){
   }


}











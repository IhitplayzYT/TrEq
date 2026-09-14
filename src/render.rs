pub mod render{
   // Use the ratatui-sci-fi widget lib for faster widgets

use std::{error::Error, io::Stdout, sync::{Arc, Mutex}};
use crossterm::{event::{DisableMouseCapture, KeyCode, KeyEvent}, execute, terminal::{LeaveAlternateScreen, disable_raw_mode}};
use ratatui::{Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::Style};
use cpal::{Host, HostId};
use crossterm::event::{self, Event};
use ratatui::{widgets::{Block, Borders, List, ListItem}};
use cpal::traits::HostTrait; 
use std::time::Duration;
use ratatui::{widgets::{Paragraph, Wrap}, style::{Color, Modifier}, text::{Line, Span}};
use crate::models::models::{App, SelectInterfaces, EqProfile, EqNode, EqBand, EqFilter, Freq};


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

   pub fn select_output_device(terminal: &mut Terminal<CrosstermBackend<Stdout>>, host: &Host) -> Result<String, Box<dyn Error>> {
      let mut state = SelectionState::new(host.output_devices()?.map(|d| d.to_string()).collect::<Vec<String>>());
      terminal.clear()?;

      loop {
         terminal.draw(|f| {
               let size = f.area();
               let block = Block::default().title("Select output device").borders(Borders::ALL);  
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
      let tick_rate = Duration::from_millis(16); // 60 FPS
      
      loop {
         terminal.draw(|f| {
               let mut app_guard = app.lock().unwrap();
               app_guard.dao.load_all_confs();
               draw_ui(f, &app_guard);
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

   fn draw_ui(f: &mut Frame, app: &App) {
      let size = f.area();
      
      let chunks = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
         ]).split(size);

      let header_text = vec![Line::from(vec![Span::styled("TrEq", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),Span::raw(" - Hyper Customizable Equalizer TUI")])];
      let header = Paragraph::new(header_text).block(Block::default().borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(header, chunks[0]);

      match app.current_interface {
         SelectInterfaces::AddProfile => draw_add_profile(f, chunks[1], app),
         SelectInterfaces::SelectProfile => draw_select_profile(f, chunks[1], app),
         SelectInterfaces::AddNode => draw_add_node(f, chunks[1], app),
         SelectInterfaces::SelectNode => draw_select_node(f, chunks[1], app),
         SelectInterfaces::AddEqBand => draw_add_band(f, chunks[1], app),
         SelectInterfaces::SelectEqBand => draw_select_band(f, chunks[1], app),
         SelectInterfaces::UpdateEqBand => draw_update_band(f, chunks[1], app),
         SelectInterfaces::SelectFilter => draw_select_filter(f, chunks[1], app),
         SelectInterfaces::ViewEqCurve => draw_eq_curve(f, chunks[1], app),
         SelectInterfaces::UpdateNode => draw_update_node(f, chunks[1], app),
         SelectInterfaces::UpdateProfile => draw_update_profile(f, chunks[1], app),
         SelectInterfaces::DeleteNode => draw_delete_node(f, chunks[1], app),
         SelectInterfaces::DeleteEqBand => draw_delete_band(f, chunks[1], app),
         SelectInterfaces::DeleteProfile => draw_delete_profile(f, chunks[1], app),
         SelectInterfaces::SearchProfile => draw_search_profile(f, chunks[1], app),
         SelectInterfaces::SaveProfile => draw_save_profile(f, chunks[1], app),
         SelectInterfaces::SaveProfiles => draw_save_profiles(f, chunks[1], app),
         SelectInterfaces::Finalize => draw_finalize(f, chunks[1], app),
         SelectInterfaces::Load => draw_load(f, chunks[1], app),
         SelectInterfaces::Unload => draw_unload(f, chunks[1], app),
         SelectInterfaces::AllProfiles => draw_allprofiles(f,chunks[1],app)
      }

      let footer_text = if let Some(msg) = &app.message {
         vec![Line::from(vec![Span::styled(format!("Interface: {} | ", app.current_interface.name()), Style::default().fg(Color::Gray)),Span::styled(msg, Style::default().fg(Color::Yellow))])]
      } else {
         vec![Line::from(vec![Span::styled(format!("Interface: {}", app.current_interface.name()), Style::default().fg(Color::Gray))])]
      };
      let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(footer, chunks[2]);

   }

   fn draw_default(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Use Tab to navigate between interfaces"),
         Line::from("Press Enter to select"),
         Line::from("Press 'q' to quit"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Help").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_add_profile(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Add New Profile"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type profile name and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Add Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_select_profile(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let items: Vec<ListItem> = app.profiles.iter().enumerate().map(|(i, p)| {
         let prefix = if i == app.selected_profile_idx { "> " } else { "  " };
         let status = if p.get_preamp().is_some() { format!(" (Preamp: {:.1}dB)", p.get_preamp().unwrap()) } else { String::new() };
         ListItem::new(format!("{}{}{} - {} bands", prefix, p.get_name(), status, p.node_count()))
      }).collect();
      let list = List::new(items).block(Block::default().title("Select Profile").borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(list, area);
   }

   fn draw_add_node(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Add New EQ Node"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type node name and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Add Node").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_select_node(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      if app.selected_profile_idx >= app.profiles.len() {
         let text = vec![Line::from("No profile selected")];
         let paragraph = Paragraph::new(text).block(Block::default().title("Select Node").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }
      let profile = &app.profiles[app.selected_profile_idx];
      let nodes: Vec<_> = profile.nodes.values().collect();
      let items: Vec<ListItem> = nodes.iter().enumerate().map(|(i, node)| {
         let node = node.lock().unwrap();
         let prefix = if i == app.selected_node_idx { "> " } else { "  " };
         let name = node.get_name().as_ref().map(|s| s.as_str()).unwrap_or("Unnamed");
         let status = if node.is_enabled() { "[Enabled]" } else { "[Disabled]" };
         ListItem::new(format!("{}{} {} - {} bands", prefix, name, status, node.band_count()))
      }).collect();
      let list = List::new(items).block(Block::default().title("Select Node").borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(list, area);
   }

   fn draw_add_band(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Add New EQ Band"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Format: freq,gain,q (e.g., 1000,3.0,1.0)"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Add Band").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_select_band(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      if app.selected_profile_idx >= app.profiles.len() {
         let text = vec![Line::from("No profile selected")];
         let paragraph = Paragraph::new(text).block(Block::default().title("Select Band").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }
      let profile = &app.profiles[app.selected_profile_idx];
      let nodes: Vec<_> = profile.nodes.values().collect();
      if app.selected_node_idx >= nodes.len() {
         let text = vec![Line::from("No node selected")];
         let paragraph = Paragraph::new(text).block(Block::default().title("Select Band").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }

      let node = nodes[app.selected_node_idx].lock().unwrap();      
      let items: Vec<ListItem> = node.bands.iter().enumerate().map(|(i, band)| {
         let prefix = if i == app.selected_band_idx { "> " } else { "  " };
         let freq = match band.freq {
            Freq::Point(f) => format!("{}Hz", f),
            Freq::Range(s, e) => format!("{}-{}Hz", s, e),
         };
         let filter = band.filter.map(|f| f.to_string()).unwrap_or("None".to_string());
         let status = if band.enabled { "[On]" } else { "[Off]" };
         ListItem::new(format!("{}{} {} {:.1}dB Q={:.1} {}", prefix, freq, filter, band.gain, band.q, status))
      }).collect();
      let list = List::new(items).block(Block::default().title("Select Band").borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(list, area);
   }

   fn draw_update_band(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Update EQ Band"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Format: freq,gain,q (e.g., 1000,3.0,1.0)"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Update Band").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_select_filter(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let filters = vec!["Peaking", "LowShelf", "HighShelf", "LowPass", "HighPass", "BandPass", "Notch","None"];
      
      let items: Vec<ListItem> = filters.iter().enumerate().map(|(i, filter)| {
         let prefix = if i == app.selected_filter_idx { "> " } else { "  " };
         ListItem::new(format!("{}{}", prefix, filter))
      }).collect();
      let list = List::new(items).block(Block::default().title("Select Filter Type").borders(Borders::ALL)).style(Style::default().fg(Color::White));
      f.render_widget(list, area);
   }

   fn draw_eq_curve(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      if app.selected_profile_idx >= app.profiles.len() {
         let text = vec![Line::from("No profile selected")];
         let paragraph = Paragraph::new(text).block(Block::default().title("EQ Curve").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }
      let profile = &app.profiles[app.selected_profile_idx];
      let nodes: Vec<_> = profile.nodes.values().collect();
      if nodes.is_empty() {
         let text = vec![Line::from("No EQ bands configured")];
         let paragraph = Paragraph::new(text).block(Block::default().title("EQ Curve").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }

      let mut all_bands = Vec::new();
      for node in &nodes {
         let node = node.lock().unwrap();
         for band in &node.bands {
            if band.enabled {
               all_bands.push(band.clone());
            }
         }
      }

      if all_bands.is_empty() {
         let text = vec![Line::from("No enabled bands")];
         let paragraph = Paragraph::new(text)
            .block(Block::default().title("EQ Curve").borders(Borders::ALL));
         f.render_widget(paragraph, area);
         return;
      }

      let mut text = vec![
         Line::from("Frequency Response"),
         Line::from(""),
         Line::from("+12dB |"),
         Line::from(" +6dB |"),
         Line::from("  0dB |----------------"),
         Line::from(" -6dB |"),
         Line::from("-12dB |"),
         Line::from(""),
         Line::from("20Hz    200Hz   2kHz    20kHz"),
         Line::from(""),
         Line::from("Active Bands:")
      ];
      for (i, band) in all_bands.iter().enumerate() {
         let freq = match band.freq {
            Freq::Point(f) => format!("{}Hz", f),
            Freq::Range(s, e) => format!("{}-{}Hz", s, e),
         };
         let filter = band.filter.map(|f| f.to_string()).unwrap_or("None".to_string());
         text.push(Line::from(format!("  {}. {} {} {:.1}dB Q={:.1}", i + 1, freq, filter, band.gain, band.q)));
      }
      let paragraph = Paragraph::new(text).block(Block::default().title("EQ Curve (Simplified)").borders(Borders::ALL)).wrap(Wrap { trim: false });
      f.render_widget(paragraph, area);
   }

   fn draw_update_node(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Update Node"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type new node name and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Update Node").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_update_profile(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Update Profile"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type new profile name and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Update Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_delete_node(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Delete Node"),
         Line::from(""),
         Line::from("Press Enter to confirm deletion"),
         Line::from("Press Esc to cancel"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Delete Node").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_delete_band(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Delete Band"),
         Line::from(""),
         Line::from("Press Enter to confirm deletion"),
         Line::from("Press Esc to cancel"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Delete Band").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_delete_profile(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Delete Profile"),
         Line::from(""),
         Line::from("Press Enter to confirm deletion"),
         Line::from("Press Esc to cancel"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Delete Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_search_profile(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Search Profile"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type search term and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Search Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_save_profile(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Save Profile"),
         Line::from(""),
         Line::from("Press Enter to save current profile"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Save Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_save_profiles(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Save All Profiles"),
         Line::from(""),
         Line::from("Press Enter to save all profiles"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Save All Profiles").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_finalize(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Finalize Profile"),
         Line::from(""),
         Line::from("This will clean up disabled nodes and bands"),
         Line::from(""),
         Line::from("Press Enter to finalize"),
         Line::from("Press Esc to cancel"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Finalize").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }


   fn draw_allprofiles(f: &mut Frame,area: Rect,app: &App){
      let mut text = vec![
         Line::from("Available Profiles: "),
         Line::from("")
      ];
      for i in &app.dao.all_confs{
         text.push(Line::from(&i[..]));         
      }
      let para = Paragraph::new(text).block(Block::default().title("All available Profiles").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(para, area);
   }

   fn draw_load(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
      let text = vec![
         Line::from("Load Profile"),
         Line::from(""),
         Line::from(format!("Buffer: {}", app.cur_buff)),
         Line::from(""),
         Line::from("Type profile name to load and press Enter"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Load Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn draw_unload(f: &mut Frame, area: ratatui::layout::Rect, _app: &App) {
      let text = vec![
         Line::from("Unload Profile"),
         Line::from(""),
         Line::from("Press Enter to unload current profile"),
         Line::from("Press Esc to cancel"),
      ];
      let paragraph = Paragraph::new(text).block(Block::default().title("Unload Profile").borders(Borders::ALL)).wrap(Wrap { trim: true });
      f.render_widget(paragraph, area);
   }

   fn handle_input(app: &mut App,key: KeyEvent){
      use KeyCode::*;      
      match key.code {
         Tab => {
            app.current_interface = app.current_interface.next();
            app.message = Some(format!("Switched to {}", app.current_interface.name()));
         },
         BackTab => {
            app.current_interface = app.current_interface.prev();
            app.message = Some(format!("Switched to {}", app.current_interface.name()));
         },
         Char(c) => {
            app.cur_buff.push(c);
            app.is_buff_dirty = true;
         },
         Backspace => {
            app.cur_buff.pop();
            app.is_buff_dirty = true;
         },
         Enter => {
            handle_enter(app);
         }
         Up => {
            handle_navigation(app, true);
         }
         Down => {
            handle_navigation(app, false);
         }
         Esc => {
            app.cur_buff.clear();
            app.is_buff_dirty = false;
            app.message = Some("Cancelled".to_string());
         }
         _ => {}
      }
   }

   fn handle_enter(app: &mut App) {
      use SelectInterfaces::*;      
      match app.current_interface {
         AddProfile => {
            if !app.cur_buff.is_empty() {
               let name = app.cur_buff.clone();
               app.profiles.push(EqProfile::new(&name, None, 2));
               app.cur_buff.clear();
               app.is_buff_dirty = false;
               app.message = Some(format!("Created profile: {}", name));
               app.current_interface = SelectProfile;
            }
         }
         AddNode => {
            if app.selected_profile_idx < app.profiles.len() && !app.cur_buff.is_empty() {
               let name = Some(app.cur_buff.clone());
               let node = EqNode::new(name);
               let profile = &mut app.profiles[app.selected_profile_idx];
               profile.add_node(None, node);
               app.cur_buff.clear();
               app.is_buff_dirty = false;
               app.message = Some("Node added".to_string());
               app.current_interface = SelectNode;
            }
         }
         AddEqBand => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() {
                  let parts: Vec<&str> = app.cur_buff.split(',').collect();
                  if parts.len() >= 3 {
                     if let (Ok(freq), Ok(gain), Ok(q)) = (
                        parts[0].trim().parse::<usize>(),
                        parts[1].trim().parse::<f32>(),
                        parts[2].trim().parse::<f64>()
                     ) {
                        let node = nodes[app.selected_node_idx].clone();
                        let filters = [EqFilter::Peaking, EqFilter::LowShelf, EqFilter::HighShelf, EqFilter::LowPass, EqFilter::HighPass, EqFilter::BandPass, EqFilter::Notch];
                        let filter = filters.get(app.selected_filter_idx).copied().unwrap_or(EqFilter::Peaking);
                        node.lock().unwrap().add_band(Freq::Point(freq), Some(filter), Some(gain), 48000, q);
                        app.cur_buff.clear();
                        app.is_buff_dirty = false;
                        app.message = Some("Band added".to_string());
                        app.current_interface = SelectEqBand;
                     }
                  }
               }
            }
         }
         SelectFilter => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() {
                  let node = nodes[app.selected_node_idx].clone();
                  let bands: Vec<_> = node.lock().unwrap().bands.iter().map(|b| b.id).collect();
                  if app.selected_band_idx < bands.len() {
                     let filters = [EqFilter::Peaking, EqFilter::LowShelf, EqFilter::HighShelf, EqFilter::LowPass, EqFilter::HighPass, EqFilter::BandPass, EqFilter::Notch];
                     let filter = filters.get(app.selected_filter_idx).copied().unwrap_or(EqFilter::Peaking);
                     let mut node = node.lock().unwrap();
                     if let Some(band) = node.get_band_mut(bands[app.selected_band_idx]) {
                        band.update_filter(Some(filter));
                     }
                     app.message = Some(format!("Filter changed to: {}", filter.to_string()));
                     app.current_interface = SelectEqBand;
                  }
               }
            }
         }
         DeleteProfile => {
            if app.selected_profile_idx < app.profiles.len() {
               app.profiles.remove(app.selected_profile_idx);
               if app.selected_profile_idx >= app.profiles.len() && !app.profiles.is_empty() {
                  app.selected_profile_idx = app.profiles.len() - 1;
               }
               app.message = Some("Profile deleted".to_string());
               app.current_interface = SelectProfile;
            }
         }
         DeleteNode => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &mut app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.keys().cloned().collect();
               if app.selected_node_idx < nodes.len() {
                  profile.deactive_node(nodes[app.selected_node_idx]);
                  app.message = Some("Node deactivated".to_string());
                  app.current_interface = SelectNode;
               }
            }
         }
         DeleteEqBand => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() {
                  let node = nodes[app.selected_node_idx].clone();
                  let bands: Vec<_> = node.lock().unwrap().bands.iter().map(|b| b.id).collect();
                  if app.selected_band_idx < bands.len() {
                     node.lock().unwrap().disable_band(bands[app.selected_band_idx]);
                     app.message = Some("Band disabled".to_string());
                     app.current_interface = SelectEqBand;
                  }
               }
            }
         }
         SaveProfile => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let fname = format!("{}.json", profile.get_name());
               if let Err(e) = app.dao.create_config(&fname, profile) {
                  app.message = Some(format!("Error saving: {}", e));
               } else {
                  app.message = Some(format!("Saved: {}", fname));
               }
            }
         }
         SaveProfiles => {
            let mut saved = 0;
            for profile in &app.profiles {
               let fname = format!("{}.json", profile.get_name());
               if app.dao.create_config(&fname, profile).is_ok() {
                  saved += 1;
               }
            }
            app.message = Some(format!("Saved {} profiles", saved));
         }
         Finalize => {
            if app.selected_profile_idx < app.profiles.len() {
               app.profiles[app.selected_profile_idx].finalize();
               app.message = Some("Profile finalized".to_string());
            }
         }
         Load => {
            if !app.cur_buff.is_empty() {
               let fname = format!("{}.json", app.cur_buff);
               if let Ok(Some(profile)) = app.dao.load_conf(&fname) {
                  app.profiles.push(profile);
                  app.cur_buff.clear();
                  app.is_buff_dirty = false;
                  app.message = Some(format!("Loaded: {}", fname));
                  app.current_interface = SelectProfile;
               } else {
                  app.message = Some(format!("Failed to load: {}", fname));
               }
            }
         }
         Unload => {
            if app.selected_profile_idx < app.profiles.len() {
               app.profiles.remove(app.selected_profile_idx);
               if app.selected_profile_idx >= app.profiles.len() && !app.profiles.is_empty() {
                  app.selected_profile_idx = app.profiles.len() - 1;
               }
               app.message = Some("Profile unloaded".to_string());
            }
         }
         UpdateProfile => {
            if app.selected_profile_idx < app.profiles.len() && !app.cur_buff.is_empty() {
               app.profiles[app.selected_profile_idx].set_name(&app.cur_buff);
               app.cur_buff.clear();
               app.is_buff_dirty = false;
               app.message = Some("Profile name updated".to_string());
            }
         }
         UpdateNode => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &mut app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() && !app.cur_buff.is_empty() {
                  nodes[app.selected_node_idx].lock().unwrap().set_name(Some(app.cur_buff.clone()));
                  app.cur_buff.clear();
                  app.is_buff_dirty = false;
                  app.message = Some("Node name updated".to_string());
               }
            }
         }
         UpdateEqBand => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() {
                  let node = nodes[app.selected_node_idx].clone();
                  let bands: Vec<_> = node.lock().unwrap().bands.iter().map(|b| b.id).collect();
                  if app.selected_band_idx < bands.len() && !app.cur_buff.is_empty() {
                     let parts: Vec<&str> = app.cur_buff.split(',').collect();
                     if parts.len() >= 3 {
                        if let (Ok(freq), Ok(gain), Ok(q)) = (parts[0].trim().parse::<usize>(),parts[1].trim().parse::<f32>(),parts[2].trim().parse::<f64>()) {
                           let mut node = node.lock().unwrap();
                           if let Some(band) = node.get_band_mut(bands[app.selected_band_idx]) {
                              band.update_freq(Freq::Point(freq));
                              band.update_gain(gain);
                              band.update_q(q);
                           }
                           app.cur_buff.clear();
                           app.is_buff_dirty = false;
                           app.message = Some("Band updated".to_string());
                        }
                     }
                  }
               }
            }
         }
         SearchProfile => {
            if !app.cur_buff.is_empty() {
               let search = app.cur_buff.to_lowercase();
               for (i, profile) in app.profiles.iter().enumerate() {
                  if profile.get_name().to_lowercase().contains(&search) {
                     app.selected_profile_idx = i;
                     app.message = Some(format!("Found: {}", profile.get_name()));
                     app.current_interface = SelectProfile;
                     return;
                  }
               }
               app.message = Some("Not found".to_string());
            }
         }
         _ => {}
      }
   }

   fn handle_navigation(app: &mut App, up: bool) {
      use SelectInterfaces::*;      
      match app.current_interface {
         SelectProfile => {
            if !app.profiles.is_empty() {
               if up {
                  app.selected_profile_idx = if app.selected_profile_idx == 0 {
                     app.profiles.len() - 1
                  } else {
                     app.selected_profile_idx - 1
                  };
               } else {
                  app.selected_profile_idx = (app.selected_profile_idx + 1) % app.profiles.len();
               }
            }
         }
         SelectNode => {
            if app.selected_profile_idx < app.profiles.len() {
               let node_count = app.profiles[app.selected_profile_idx].node_count();
               if node_count > 0 {
                  if up {
                     app.selected_node_idx = if app.selected_node_idx == 0 {
                        node_count - 1
                     } else {
                        app.selected_node_idx - 1
                     };
                  } else {
                     app.selected_node_idx = (app.selected_node_idx + 1) % node_count;
                  }
               }
            }
         }
         SelectEqBand => {
            if app.selected_profile_idx < app.profiles.len() {
               let profile = &app.profiles[app.selected_profile_idx];
               let nodes: Vec<_> = profile.nodes.values().collect();
               if app.selected_node_idx < nodes.len() {
                  let band_count = nodes[app.selected_node_idx].lock().unwrap().band_count();
                  if band_count > 0 {
                     if up {
                        app.selected_band_idx = if app.selected_band_idx == 0 {
                           band_count - 1
                        } else {
                           app.selected_band_idx - 1
                        };
                     } else {
                        app.selected_band_idx = (app.selected_band_idx + 1) % band_count;
                     }
                  }
               }
            }
         }
         SelectFilter => {
            let filter_count = 8;
            if up {
               app.selected_filter_idx = if app.selected_filter_idx == 0 {
                  filter_count - 1
               } else {
                  app.selected_filter_idx - 1
               };
            } else {
               app.selected_filter_idx = (app.selected_filter_idx + 1) % filter_count;
            }
         }
         _ => {}
      }
   }
}

use console_engine::crossterm::event::KeyEvent;
use console_engine::screen::Screen;
use console_engine::forms::{FormField, FormOptions, FormValue, Form, Text};
use console_engine::events::Event;
use console_engine::rect_style::BorderStyle;
use console_engine::{pixel, KeyCode, MouseButton, ConsoleEngine, KeyModifiers};
use uuid::Uuid;
use sqlx::postgres::PgPoolOptions;

#[derive(sqlx::FromRow, Debug)]
struct Todo {
    todo_id: Uuid,
    name: String,
    description: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    time_cost: Option<i32>,
    done: bool,
    // Add recurring to this, also how often to recur
}

impl Todo {
    fn new() -> Todo {
        return Todo { todo_id: Uuid::new_v4(), name: "placeholder".to_string(), description: "Placeholder decription".to_string(), start_time: Some(1), end_time: Some(2), time_cost: Some(3), done: false };
    }
}

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Position {
    fn new() -> Position {
        return Position { x: 0, y: 0, width: 30, height: 1};
    }

    fn default(x_axis: i32, y_axis: i32, width: i32, height: i32) -> Position {
        return Position { x: x_axis, y: y_axis, width: width, height: height};
    }
}

#[derive(Debug)]
struct TodoBlock {
    todo: Todo,
    position: Position,
}

impl TodoBlock {
    fn new(x: i32, y: i32) -> TodoBlock {
        return TodoBlock { todo: Todo::new(), position: Position::default(x, y, 30, 1) };
    }
}

#[derive(Debug)]
struct Day {
    todo_blocks: Vec<TodoBlock>,
    position: Position,
}

impl Day {
    fn new(x: i32, y:i32) -> Day {
        return Day { todo_blocks: Vec::new(), position: Position::default(x, y, 30, 10)};
    }

    fn re_order(&mut self) {
        let mut index = 0;
        while index < self.todo_blocks.len() {
            self.todo_blocks[index].position.y = (self.position.y+index as i32) +1;
            index = index +1;
        }
    }
}

fn generate_todo() -> Todo {
    let idd: Uuid = Uuid::new_v4();
    let todo: Todo = Todo { todo_id: idd, name: "deez".to_string(), description: "description".to_string(), start_time: Some(1), end_time: Some(2), time_cost: Some(3), done: false };
    return todo; 
}

#[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres@localhost/test").await?;
    let width: i32 = 300;
    let height: i32 = 80;
    let mut scr = Screen::new(width as u32, height as u32);
    let horizontal_character = pixel::pxl('-');
    let vertical_character = pixel::pxl('|');
    let mut index = 0;
    
    let rows = sqlx::query_as::<_,Todo>("SELECT * FROM todo").fetch_all(&pool).await?;

    for item in rows {
        println!("{:?}", item);
    }

    while index < (width/10)  {
        let todo = generate_todo();
        //sqlx::query("INSERT INTO todo (name, description) VALUES ($1,$2)").bind(&todo.name).bind(&todo.description).execute(&pool).await?;
        scr.h_line(0, 10 + index*10, width-1, horizontal_character);
        scr.v_line(30 + index*30, 0, height-1, vertical_character);
        scr.print(0, 1 + index*10, &todo.todo_id.to_string());
        scr.print(0, 2 + index*10, &todo.name.as_str());
        scr.print(0, 3 + index*10, &todo.description.as_str());
        index = index + 1;
    }

    draw();
    //scr.draw();
    // print the screen to the terminal
    Ok(())
}



fn draw() {
    let mut engine = console_engine::ConsoleEngine::init_fill(30).unwrap();
    let mut days: Vec<Vec<Day>> = vec!(Vec::new());
    let mut other: Vec<Day> = Vec::new();
    let mut unassigned_todos: Day = Day { todo_blocks: Vec::new(), position: Position { x: 251, y: 1, width: 60, height: 30 }};
    let mut done_todos: Day = Day { todo_blocks: Vec::new(), position: Position { x: 251, y: 33, width: 60, height: 30 }};
    let mut overlay: bool = false;

    unassigned_todos.todo_blocks.push(TodoBlock { todo: Todo::new(), position: Position { x: 252, y: 2,  width: 58, height: 1}});
    done_todos.todo_blocks.push(TodoBlock { todo: Todo::new(), position: Position { x: 252, y: 34,  width: 58, height: 1}});
    other.push(unassigned_todos);
    other.push(done_todos);

    let mut pos_in_days: Option<(i32,i32,i32)> = None;

    let mut debug = -1; 
    let mut debug_x_rel = -1; 
    let mut debug_y = -1; 
    let mut debug_y_rel = -1; 
    let mut debug_pos = -1;

    for x_axis in 0..7 {
        let mut day_list: Vec<Day> = Vec::new();
        for y_axis in 0..7 {
            let mut day = Day::new(x_axis*30, y_axis*10);
            day.todo_blocks.push(TodoBlock::new(1+ x_axis*30, 1+ y_axis*10));
            day_list.push(day);
        }
        days.push(day_list);
    }
    days.push(other);

    let mut form = Form::new(30, 10, FormOptions {..Default::default()});
    form.build_field::<Text>("name", FormOptions { label: Some("Name"), ..Default::default()});
    let mut f_text = console_engine::forms::Text::new(30, FormOptions::default());
    //f_text.set_active(true);

    loop {
        engine.wait_frame();

        if engine.is_key_pressed(KeyCode::Char('q')) {
            break;
        }

        if engine.is_key_pressed(KeyCode::Esc) {
            overlay=false;
        }

        if engine.is_key_pressed_with_modifier(KeyCode::Char('n'), KeyModifiers::CONTROL) {
            overlay = true;
        }

        engine.clear_screen();

        for list in days.iter() {
            for day in list.iter() {
                draw_day(&mut engine, &day);
            }
        }

        engine.print(251, 0, &"Unassigned");
        engine.print(251, 32, &"Done");

        if overlay {
            form.set_active(true);
            while overlay {
                let event = engine.poll();
                match event {
                    Event::Frame => {
                        engine.rect(40, 15, 4*40, 4*15, pixel::pxl('='));
                        engine.fill_rect(41, 16, 4*40-1, 4*15-1, pixel::pxl(' '));
                        engine.print(2*40-4, 16, "New todo");

                        engine.print(42, 40, "Naam: ");
                        engine.rect_border(48, 39, 80, 41, BorderStyle::new_light());
                        engine.print_screen(49, 40, form.draw((engine.frame_count % 8 > 3) as usize));
                        engine.draw(); 
                    }
                    Event::Key(KeyEvent {
                        code:  KeyCode::Esc,
                        modifiers: _,
                    }) => {
                        form.set_active(false);
                        overlay = false;
                        break;
                    }
                    event => form.handle_event(event)
                }
            }
        }



        let mouse_pos = engine.get_mouse_press(MouseButton::Left);
        if let Some(mouse_pos) = mouse_pos {
            pos_in_days = check_mouse_position_days(mouse_pos.0 as i32, mouse_pos.1 as i32, &mut days);
        }

        if pos_in_days.is_none() {
            engine.draw();
            continue;
        }


        let pos = pos_in_days.unwrap();
        if pos.2 == -1 {
            engine.draw();
            continue;
        }

        let mouse_pos = engine.get_mouse_held(MouseButton::Left);
        if let Some(mouse_pos) = mouse_pos {
            get_position(&mut days, pos.0, pos.1, pos.2).x = mouse_pos.0 as i32;
            get_position(&mut days, pos.0, pos.1, pos.2).y = mouse_pos.1 as i32;
        }

        let mouse_pos = engine.get_mouse_released(MouseButton::Left);
        if let Some(mouse_pos) = mouse_pos{
            // Check where the new position is, and move to that position
            let new_pos;

            match check_mouse_position_days(mouse_pos.0 as i32, mouse_pos.1 as i32, &mut days) {
                Some(x) => new_pos = x,
                None => {engine.draw(); pos_in_days = None; continue;}
            }

            if pos.0 != new_pos.0 || pos.1 != new_pos.1 {
                get_position(&mut days, pos.0, pos.1, pos.2).x = days.get(new_pos.0 as usize).unwrap().get(new_pos.1 as usize).unwrap().position.x+1;
                get_position(&mut days, pos.0, pos.1, pos.2).y = days.get(new_pos.0 as usize).unwrap().get(new_pos.1 as usize).unwrap().position.y+ days.get(new_pos.0 as usize).unwrap().get(new_pos.1 as usize).unwrap().todo_blocks.len() as i32 +1; 
                let mut old_day = days.get_mut(pos.0 as usize).unwrap().get_mut(pos.1 as usize).unwrap();
                let todo_to_move = old_day.todo_blocks.remove(pos.2 as usize);
                old_day.re_order();
                let mut new_day = days.get_mut(new_pos.0 as usize).unwrap().get_mut(new_pos.1 as usize).unwrap();
                new_day.todo_blocks.push(todo_to_move);
            }
            pos_in_days = None;
        }

        engine.draw();
    }
}

fn draw_day(engine: &mut ConsoleEngine, day: &Day) {
    engine.rect(day.position.x, day.position.y, day.position.x + day.position.width, day.position.height + day.position.y, pixel::pxl('#'));
    draw_todo_blocks(engine, &day.todo_blocks);
}

fn check_mouse_position_days(mouse_x: i32, mouse_y: i32, days: &mut Vec<Vec<Day>>) -> Option<(i32,i32,i32)> {
    let relative_x = mouse_x /30;
    let relative_y = mouse_y /10;
    if relative_x < days.len() as i32  -1 {
        let day_list = days.get((relative_x +1) as usize).unwrap();
        if relative_y < day_list.len() as i32 {
            let day = day_list.get(relative_y as usize).unwrap();
            let relative_todo_position = -1 * (day.position.y - mouse_y as i32) % 10 -1;
            if day.todo_blocks.get(relative_todo_position as usize).is_some() {
                return Some((relative_x +1, relative_y, relative_todo_position));
            }
            return Some((relative_x +1, relative_y, -1));
        }
    } else {
        let mut index: usize = 0;
        let other = days.get(days.len()-1).unwrap();
        while index < other.len() { 
            let day = other.get(index).unwrap();
            if day.position.x < mouse_x && day.position.x + day.position.width > mouse_x && day.position.y < mouse_y && day.position.y + day.position.height > mouse_y {
                let relative_todo_position = -1 * (day.position.y - mouse_y as i32) % 10 -1;
                if day.todo_blocks.get(relative_todo_position as usize).is_some() {
                    return Some((days.len() as i32 -1, index as i32, relative_todo_position));
                }
                return Some((days.len() as i32 -1, index as i32, -1));
            }
            index = index +1;
        }
        return None;
    }
    return None;
}

fn draw_overlay(engine: &mut ConsoleEngine) {
}

fn check_mouse_position_other(mouse_x: i32, mouse_y: i32, other: &Vec<Day>) -> Option<(usize, i32)>{
    let mut index: usize = 0;
    while index < other.len() { 
        let day = other.get(index).unwrap();
        if day.position.x < mouse_x && day.position.x + day.position.width > mouse_x && day.position.y < mouse_y && day.position.y + day.position.height > mouse_y {
            let relative_todo_position = -1 * (day.position.y - mouse_y as i32) % 10 -1;
            if day.todo_blocks.get(relative_todo_position as usize).is_some() {
                return Some((index, relative_todo_position));
            }
        }
        index = index +1;
    }
    return None;
}




fn draw_todo_blocks(engine: &mut ConsoleEngine, blocks: &Vec<TodoBlock>) {
   for todo_block in blocks.iter() {
       if (todo_block.todo.todo_id.to_string().len() as i32) < todo_block.position.width -2 {
           engine.print(todo_block.position.x, todo_block.position.y, &todo_block.todo.todo_id.to_string());
       } else {
           let test = todo_block.todo.todo_id.to_string();
           let (left, _) = test.split_at(todo_block.position.width as usize -5);
           let formated_name = format!("{}{}", left, "...");
           engine.print(todo_block.position.x, todo_block.position.y, &formated_name);
       }
   }
}

fn check_in_day(days: &mut Vec<Vec<Day>>, day_x_pos: i32, day_y_pos: i32) -> bool {
    return false;
}


fn get_position(days: &mut Vec<Vec<Day>>, day_x: i32, day_y: i32, todo_pos: i32) -> &mut Position {
    return &mut days.get_mut(day_x as usize).unwrap().get_mut(day_y as usize).unwrap().todo_blocks.get_mut(todo_pos as usize).unwrap().position;
}

fn get_position_other(days: &mut Vec<Day>, index: usize, todo_pos: i32) -> &mut Position {
    return &mut days.get_mut(index).unwrap().todo_blocks.get_mut(todo_pos as usize).unwrap().position;
}

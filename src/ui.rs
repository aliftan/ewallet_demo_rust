use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, AppState};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let title = Paragraph::new("E-Wallet Demo")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match app.current_state {
        AppState::MainMenu => draw_main_menu(f, chunks[1]),
        AppState::Login => draw_login(f, app, chunks[1]),
        AppState::CreateAccount => draw_create_account(f, app, chunks[1]),
        AppState::LoggedIn => draw_logged_in(f, app, chunks[1]),
        AppState::Deposit => draw_deposit(f, app, chunks[1]),
        AppState::Withdraw => draw_withdraw(f, app, chunks[1]),
        AppState::Transfer => draw_transfer(f, app, chunks[1]),
        AppState::ViewTransactions => draw_transactions(f, app, chunks[1]),
    }

    draw_messages(f, app);
}

fn draw_main_menu<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let items = vec![
        ListItem::new("1. Login"),
        ListItem::new("2. Create Account"),
        ListItem::new("q. Quit"),
    ];

    let menu = List::new(items)
        .block(Block::default().title("Main Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(menu, area);
}

fn draw_login<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Username"),
        );
    f.render_widget(input, area);
}

fn draw_create_account<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter New Username"),
        );
    f.render_widget(input, area);
}

fn draw_logged_in<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let balance = app.get_balance().unwrap_or(0.0);
    let account_name = app.get_current_user().unwrap_or("Unknown");
    let items = vec![
        ListItem::new(format!("Account: {}", account_name)),
        ListItem::new(format!("Current Balance: ${:.2}", balance)),
        ListItem::new("1. Deposit"),
        ListItem::new("2. Withdraw"),
        ListItem::new("3. Transfer"),
        ListItem::new("4. View Transactions"),
        ListItem::new("5. Logout"),
    ];

    let menu = List::new(items)
        .block(Block::default().title("Account Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(menu, area);
}

fn draw_deposit<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Deposit Amount"),
        );
    f.render_widget(input, area);
}

fn draw_withdraw<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Withdrawal Amount"),
        );
    f.render_widget(input, area);
}

fn draw_transfer<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let title = if app.transfer_recipient.is_none() {
        "Enter Recipient Username"
    } else {
        "Enter Transfer Amount"
    };
    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(input, area);
}

fn draw_transactions<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let transactions = app.get_transactions().unwrap_or_default();
    let items: Vec<ListItem> = transactions
        .iter()
        .map(|t| {
            let amount = t.get("amount").unwrap_or(&String::from("0")).clone();
            let transaction_type = t.get("type").unwrap_or(&String::from("Unknown")).clone();
            let recipient = t.get("recipient").unwrap_or(&String::from("")).clone();
            let sender = t.get("sender").unwrap_or(&String::from("")).clone();
            let previous_balance = t
                .get("previous_balance")
                .unwrap_or(&String::from("0"))
                .clone();
            let new_balance = t.get("new_balance").unwrap_or(&String::from("0")).clone();
            let timestamp = t.get("timestamp").unwrap_or(&String::from("")).clone();

            let description = match transaction_type.as_str() {
                "deposit" => format!("Deposit: ${}", amount),
                "withdraw" => format!("Withdrawal: ${}", amount),
                "transfer_out" => format!("Transfer: ${} to {}", amount, recipient),
                "transfer_in" => format!("Received: ${} from {}", amount, sender),
                _ => format!("Unknown transaction: ${}", amount),
            };

            ListItem::new(vec![
                Spans::from(description),
                Spans::from(format!(
                    "  Previous Balance: ${} | New Balance: ${}",
                    previous_balance, new_balance
                )),
                Spans::from(Span::styled(
                    format!("  {}", timestamp),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let transactions_list = List::new(items)
        .block(
            Block::default()
                .title("Recent Transactions")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(transactions_list, area);
}

fn draw_messages<B: Backend>(f: &mut Frame<B>, app: &App) {
    if let Some((message, _)) = app.messages.last() {
        let message_area = Rect::new(10, f.size().height - 4, f.size().width - 20, 3);
        let message_widget = Paragraph::new(message.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(message_widget, message_area);
    }
}

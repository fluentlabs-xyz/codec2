use bytes::{Buf, BufMut, BytesMut};
use prettytable::color::{BLUE, GREEN, MAGENTA, RED, WHITE};
use prettytable::Attr;
use prettytable::{Cell, Row, Table};

fn create_buffer_debug_table(buf: &BytesMut, header_size: usize) -> Table {
    let mut table = Table::new();

    let body_size = buf.len().saturating_sub(header_size);

    // Типы (первая строка)
    let mut type_row = Row::new(vec![Cell::new("Type")]);
    if header_size > 0 {
        type_row.add_cell(
            Cell::new("header")
                .with_style(Attr::ForegroundColor(GREEN))
                .with_hspan(header_size),
        );
    }
    if body_size > 0 {
        type_row.add_cell(
            Cell::new("body")
                .with_style(Attr::ForegroundColor(BLUE))
                .with_hspan(body_size),
        );
    }
    if buf.capacity() > buf.len() {
        type_row.add_cell(
            Cell::new("unused")
                .with_style(Attr::ForegroundColor(MAGENTA))
                .with_hspan(buf.capacity() - buf.len()),
        );
    }
    table.add_row(type_row);

    // Индексы
    let mut index_row = Row::new(vec![Cell::new("Index")]);
    index_row.extend((0..buf.capacity()).map(|i| {
        let cell = Cell::new(&format!("{:02X}", i));
        if i % 8 == 0 {
            cell.with_style(Attr::Bold)
                .with_style(Attr::BackgroundColor(WHITE))
                .with_style(Attr::ForegroundColor(RED))
        } else {
            cell
        }
    }));
    table.add_row(index_row);

    // Значения
    let mut value_row = Row::new(vec![Cell::new("Value")]);
    value_row.extend(buf.iter().enumerate().map(|(i, &b)| {
        let mut cell = Cell::new(&format!("{:02X}", b));
        if i % 8 == 0 {
            cell = cell.with_style(Attr::Bold);
        }
        if b == 0 {
            cell = cell.with_style(Attr::ForegroundColor(RED));
        } else if i < header_size {
            cell = cell.with_style(Attr::ForegroundColor(GREEN));
        } else {
            cell = cell.with_style(Attr::ForegroundColor(BLUE));
        }
        cell
    }));
    value_row.extend((buf.len()..buf.capacity()).map(|i| {
        let cell = Cell::new("--");
        if i % 8 == 0 {
            cell.with_style(Attr::Bold)
        } else {
            cell
        }
    }));
    table.add_row(value_row);

    table
}

pub fn print_buffer_debug(buf: &BytesMut, header_size: usize) {
    let body_size = buf.len().saturating_sub(header_size);
    println!(
        "Header Size: {} | Body Size: {} | Capacity: {} | Length: {}",
        header_size,
        body_size,
        buf.capacity(),
        buf.len()
    );

    let table = create_buffer_debug_table(buf, header_size);
    table.printstd();
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_print_buffer_debug() {
    //     let mut buffer = BytesMut::with_capacity(32);
    //     for i in 0..26 {
    //         buffer.put_u8(i);
    //     }

    //     // print_buffer_debug(&buffer, 4);
    // }
}

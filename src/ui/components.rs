// Reusable UI components

use rxtui::prelude::*;
use crate::app::{AppMsg, AppState, InputMode};

/// Render an input dialog for creating projects or tasks
pub fn render_input_dialog(ctx: &Context, state: &AppState, title: &str, label: &str) -> Node {
    node! {
        div(
            bg: "#2E3440",
            pad: 3,
            w_frac: 1.0,
            h_frac: 1.0,
            align: center,
            focusable,
            @key(esc): ctx.handler(AppMsg::CancelAction),
            @key(enter): ctx.handler(AppMsg::ConfirmAction),
            @key(backspace): ctx.handler(AppMsg::InputBackspace),
            @key(delete): ctx.handler(AppMsg::InputBackspace),
            // Letters a-z
            @char('a'): ctx.handler(AppMsg::InputChar('a')),
            @char('b'): ctx.handler(AppMsg::InputChar('b')),
            @char('c'): ctx.handler(AppMsg::InputChar('c')),
            @char('d'): ctx.handler(AppMsg::InputChar('d')),
            @char('e'): ctx.handler(AppMsg::InputChar('e')),
            @char('f'): ctx.handler(AppMsg::InputChar('f')),
            @char('g'): ctx.handler(AppMsg::InputChar('g')),
            @char('h'): ctx.handler(AppMsg::InputChar('h')),
            @char('i'): ctx.handler(AppMsg::InputChar('i')),
            @char('j'): ctx.handler(AppMsg::InputChar('j')),
            @char('k'): ctx.handler(AppMsg::InputChar('k')),
            @char('l'): ctx.handler(AppMsg::InputChar('l')),
            @char('m'): ctx.handler(AppMsg::InputChar('m')),
            @char('n'): ctx.handler(AppMsg::InputChar('n')),
            @char('o'): ctx.handler(AppMsg::InputChar('o')),
            @char('p'): ctx.handler(AppMsg::InputChar('p')),
            @char('q'): ctx.handler(AppMsg::InputChar('q')),
            @char('r'): ctx.handler(AppMsg::InputChar('r')),
            @char('s'): ctx.handler(AppMsg::InputChar('s')),
            @char('t'): ctx.handler(AppMsg::InputChar('t')),
            @char('u'): ctx.handler(AppMsg::InputChar('u')),
            @char('v'): ctx.handler(AppMsg::InputChar('v')),
            @char('w'): ctx.handler(AppMsg::InputChar('w')),
            @char('x'): ctx.handler(AppMsg::InputChar('x')),
            @char('y'): ctx.handler(AppMsg::InputChar('y')),
            @char('z'): ctx.handler(AppMsg::InputChar('z')),
            // Letters A-Z
            @char('A'): ctx.handler(AppMsg::InputChar('A')),
            @char('B'): ctx.handler(AppMsg::InputChar('B')),
            @char('C'): ctx.handler(AppMsg::InputChar('C')),
            @char('D'): ctx.handler(AppMsg::InputChar('D')),
            @char('E'): ctx.handler(AppMsg::InputChar('E')),
            @char('F'): ctx.handler(AppMsg::InputChar('F')),
            @char('G'): ctx.handler(AppMsg::InputChar('G')),
            @char('H'): ctx.handler(AppMsg::InputChar('H')),
            @char('I'): ctx.handler(AppMsg::InputChar('I')),
            @char('J'): ctx.handler(AppMsg::InputChar('J')),
            @char('K'): ctx.handler(AppMsg::InputChar('K')),
            @char('L'): ctx.handler(AppMsg::InputChar('L')),
            @char('M'): ctx.handler(AppMsg::InputChar('M')),
            @char('N'): ctx.handler(AppMsg::InputChar('N')),
            @char('O'): ctx.handler(AppMsg::InputChar('O')),
            @char('P'): ctx.handler(AppMsg::InputChar('P')),
            @char('Q'): ctx.handler(AppMsg::InputChar('Q')),
            @char('R'): ctx.handler(AppMsg::InputChar('R')),
            @char('S'): ctx.handler(AppMsg::InputChar('S')),
            @char('T'): ctx.handler(AppMsg::InputChar('T')),
            @char('U'): ctx.handler(AppMsg::InputChar('U')),
            @char('V'): ctx.handler(AppMsg::InputChar('V')),
            @char('W'): ctx.handler(AppMsg::InputChar('W')),
            @char('X'): ctx.handler(AppMsg::InputChar('X')),
            @char('Y'): ctx.handler(AppMsg::InputChar('Y')),
            @char('Z'): ctx.handler(AppMsg::InputChar('Z')),
            // Numbers
            @char('0'): ctx.handler(AppMsg::InputChar('0')),
            @char('1'): ctx.handler(AppMsg::InputChar('1')),
            @char('2'): ctx.handler(AppMsg::InputChar('2')),
            @char('3'): ctx.handler(AppMsg::InputChar('3')),
            @char('4'): ctx.handler(AppMsg::InputChar('4')),
            @char('5'): ctx.handler(AppMsg::InputChar('5')),
            @char('6'): ctx.handler(AppMsg::InputChar('6')),
            @char('7'): ctx.handler(AppMsg::InputChar('7')),
            @char('8'): ctx.handler(AppMsg::InputChar('8')),
            @char('9'): ctx.handler(AppMsg::InputChar('9')),
            // Common symbols
            @char(' '): ctx.handler(AppMsg::InputChar(' ')),
            @char('-'): ctx.handler(AppMsg::InputChar('-')),
            @char('_'): ctx.handler(AppMsg::InputChar('_')),
            @char('.'): ctx.handler(AppMsg::InputChar('.')),
            @char(','): ctx.handler(AppMsg::InputChar(',')),
            @char(':'): ctx.handler(AppMsg::InputChar(':')),
            @char(';'): ctx.handler(AppMsg::InputChar(';')),
            @char('!'): ctx.handler(AppMsg::InputChar('!')),
            @char('?'): ctx.handler(AppMsg::InputChar('?')),
            @char('('): ctx.handler(AppMsg::InputChar('(')),
            @char(')'): ctx.handler(AppMsg::InputChar(')')),
            @char('['): ctx.handler(AppMsg::InputChar('[')),
            @char(']'): ctx.handler(AppMsg::InputChar(']')),
            @char('{'): ctx.handler(AppMsg::InputChar('{')),
            @char('}'): ctx.handler(AppMsg::InputChar('}')),
            @char('/'): ctx.handler(AppMsg::InputChar('/')),
            @char('\\'): ctx.handler(AppMsg::InputChar('\\')),
            @char('@'): ctx.handler(AppMsg::InputChar('@')),
            @char('#'): ctx.handler(AppMsg::InputChar('#')),
            @char('$'): ctx.handler(AppMsg::InputChar('$')),
            @char('%'): ctx.handler(AppMsg::InputChar('%')),
            @char('&'): ctx.handler(AppMsg::InputChar('&')),
            @char('*'): ctx.handler(AppMsg::InputChar('*')),
            @char('+'): ctx.handler(AppMsg::InputChar('+')),
            @char('='): ctx.handler(AppMsg::InputChar('=')),
            @char('<'): ctx.handler(AppMsg::InputChar('<')),
            @char('>'): ctx.handler(AppMsg::InputChar('>')),
            @char('|'): ctx.handler(AppMsg::InputChar('|')),
            @char('~'): ctx.handler(AppMsg::InputChar('~')),
            @char('`'): ctx.handler(AppMsg::InputChar('`')),
            @char('\''): ctx.handler(AppMsg::InputChar('\'')),
            @char('\"'): ctx.handler(AppMsg::InputChar('\"'))
        ) [
            div(
                bg: "#3B4252",
                border: "#88C0D0",
                pad: 3,
                gap: 2,
                dir: vertical,
                w: 60,
                align: center
            ) [
                // Title
                text(title, color: "#88C0D0", bold),

                spacer(1),

                // Label
                text(label, color: "#D8DEE9", align: left),

                // Input field
                div(
                    bg: "#2E3440",
                    border: "#4C566A",
                    pad: 1,
                    pad_h: 2,
                    w_frac: 1.0
                ) [
                    richtext [
                        text(&state.input_buffer, color: "#ECEFF4"),
                        text("_", color: "#ECEFF4")
                    ]
                ],

                spacer(1),

                // Instructions
                div(
                    bg: "#434C5E",
                    border: "#4C566A",
                    pad: 1,
                    pad_h: 2,
                    w_frac: 1.0
                ) [
                    richtext(align: center) [
                        text("Enter", color: "#A3BE8C", bold),
                        text(" confirm  ", color: "#D8DEE9"),
                        text("ESC", color: "#BF616A", bold),
                        text(" cancel", color: "#D8DEE9")
                    ]
                ]
            ]
        ]
    }
}

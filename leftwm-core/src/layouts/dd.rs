use crate::models::Tag;
use crate::models::Window;
use crate::models::Workspace;

/// Layout which splits the workspace into N columns, and then splits each column into rows.
/// Example arrangement (4 windows):
/// ```text
/// +---+---+
/// |   |   |
/// +---+---+
/// |   |   |
/// +---+---+
/// ```
/// or with 8 windows:
/// ```text
/// +---+---+---+
/// |   |   |   |
/// |   +---+---+
/// +---+   |   |
/// |   +---+---+
/// |   |   |   |
/// +---+---+---+
/// ```
/// The above is the default GridHorizontal layout.
/// For DD mode, one chat window is considered as two virtual windows, and must be matched to one video window.
/// Then two vertically aligned windows are further merged to one.
/// 1 chat + 1 video (1 * 2 + 1 = 3):
/// ```text
/// +---+---+    +---+---+
/// |   |   |    |   |   |
/// +---+   | -> |   |   |
/// |   |   |    |   |   |
/// +---+---+    +---+---+
/// or with 3 chats + 4 videos (3 * 2 + 4 = 10):
/// +---+---+---+---+    +---+---+---+---+
/// |   |   |   |   |    |   |   |   |   |
/// |   |   +---+---+    |   |   |   +---+
/// +---+---+   |   | -> |   |   |   |   |
/// |   |   +---+---+    |   |   +---+---+
/// |   |   |   |   |    |   |   |   |   |
/// +---+---+---+---+    +---+---+---+---+
///
pub fn update(workspace: &Workspace, tag: &Tag, windows: &mut [&mut Window]) {
    let window_count = windows.len() as i32;
    let chat_window_count = window_count / 2;
    let video_window_count = if window_count % 2 == 1 {
        window_count / 2 + 1
    } else {
        window_count / 2
    };
    let virtual_window_count = chat_window_count * 2 + video_window_count;

    // choose the number of columns so that we get close to an even NxN grid.
    let num_cols = (virtual_window_count as f32).sqrt().ceil() as i32;

    let mut iter = windows.iter_mut().enumerate().peekable();
    let mut remaining_virtual_windows = virtual_window_count;
    let mut remaining_chat_windows = chat_window_count;
    let mut remaining_video_windows = video_window_count;
    for col in 0..num_cols {
        let iter_peek = iter.peek().map(|x| x.0).unwrap_or_default() as i32;
        let remaining_columns = num_cols - col;
        let num_virtual_rows_in_this_col = remaining_virtual_windows / remaining_columns;
        let num_chat_rows_in_this_col = std::cmp::min(remaining_chat_windows, num_virtual_rows_in_this_col / 2);
        let num_video_rows_in_this_col = num_virtual_rows_in_this_col - num_chat_rows_in_this_col * 2;

        let virtual_win_height = workspace.height() / num_virtual_rows_in_this_col;
        let chat_win_height = virtual_win_height * 2;
        let video_win_height = virtual_win_height;
        let win_width = workspace.width_limited(num_cols as usize) / num_cols;

        let pos_x = if tag.flipped_horizontal {
            num_cols - col - 1
        } else {
            col
        };

        // set chat windows
        for row in 0..num_chat_rows_in_this_col {
            let Some((_idx, win)) = iter.next() else {
                return
            };
            win.set_height(chat_win_height);
            win.set_width(win_width);

            let pos_y = if tag.flipped_vertical {
                num_virtual_rows_in_this_col - row - 1
            } else {
                row
            };

            win.set_x(workspace.x_limited(num_cols as usize) + win_width * pos_x);
            win.set_y(workspace.y() + chat_win_height * pos_y);
            remaining_virtual_windows = remaining_virtual_windows - 2;
            remaining_chat_windows = remaining_chat_windows - 1;
        }

        // set video windows
        for row in 0..num_video_rows_in_this_col {
            let Some((_idx, win)) = iter.next() else {
                return
            };
            win.set_height(video_win_height);
            win.set_width(win_width);

            let pos_y = if tag.flipped_vertical {
                num_virtual_rows_in_this_col - num_chat_rows_in_this_col - row - 1
            } else {
                row + num_chat_rows_in_this_col * 2
            };

            win.set_x(workspace.x_limited(num_cols as usize) + win_width * pos_x);
            win.set_y(workspace.y() + video_win_height * pos_y);
            remaining_virtual_windows = remaining_virtual_windows - 1;
            remaining_video_windows = remaining_video_windows - 1;
        }
    }
}

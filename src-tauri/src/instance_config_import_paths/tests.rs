use super::path_text::clean_windows_path_text;

#[test]
fn hides_windows_verbatim_prefix_from_preview_paths() {
    assert_eq!(
        clean_windows_path_text(r"\\?\D:\Game\ASA-SERVER\ASA-01"),
        r"D:\Game\ASA-SERVER\ASA-01"
    );
    assert_eq!(
        clean_windows_path_text(r"\\?\UNC\server\share\ASA-01"),
        r"\\server\share\ASA-01"
    );
}

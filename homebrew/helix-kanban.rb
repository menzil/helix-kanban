class HelixKanban < Formula
  desc "Terminal-based kanban board with file-based storage and Helix-style keybindings"
  homepage "https://github.com/menzil/helix-kanban"
  version "0.2.20"
  license "MIT OR Apache-2.0"

  depends_on "rust" => :build

  # Use local source for testing
  if OS.mac?
    url "file:///Users/px/Documents/golden/kanban"
    sha256 "SKIP"
  elsif OS.linux?
    url "file:///Users/px/Documents/golden/kanban"
    sha256 "SKIP"
  end

  def install
    system "cargo", "install", "--locked", *std_cargo_args
  end

  test do
    assert_match "helix-kanban", shell_output("#{bin}/hxk --version")
  end
end

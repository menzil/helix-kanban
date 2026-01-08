class HelixKanban < Formula
  desc "Terminal-based kanban board with file-based storage and Helix-style keybindings"
  homepage "https://github.com/menzil/helix-kanban"
  version "0.2.21"
  license "MIT OR Apache-2.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/menzil/helix-kanban/releases/download/v0.2.21/hxk-macos-aarch64.tar.gz"
      sha256 "PUT_ARM64_SHA256_HERE"
    else
      url "https://github.com/menzil/helix-kanban/releases/download/v0.2.21/hxk-macos-x86_64.tar.gz"
      sha256 "PUT_X86_64_SHA256_HERE"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/menzil/helix-kanban/releases/download/v0.2.21/hxk-linux-aarch64.tar.gz"
      sha256 "PUT_LINUX_ARM64_SHA256_HERE"
    else
      url "https://github.com/menzil/helix-kanban/releases/download/v0.2.21/hxk-linux-x86_64.tar.gz"
      sha256 "PUT_LINUX_X86_64_SHA256_HERE"
    end
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "hxk-macos-aarch64" => "hxk"
      else
        bin.install "hxk-macos-x86_64" => "hxk"
      end
    elsif OS.linux?
      if Hardware::CPU.arm?
        bin.install "hxk-linux-aarch64" => "hxk"
      else
        bin.install "hxk-linux-x86_64" => "hxk"
      end
    end
  end

  test do
    assert_match "helix-kanban", shell_output("#{bin}/hxk --version")
  end
end

class ZshHighlighter < Formula
  desc "Minimal, ultra-fast zsh syntax highlighter written in Rust"
  homepage "https://github.com/EshginGuluzade/zsh-highlighter"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/zsh-highlighter"
    (share/"zsh-highlighter").install "plugin/zsh-highlighter.zsh"
  end

  def caveats
    <<~EOS
      To activate zsh-highlighter, add the following to your ~/.zshrc:

        source "$(brew --prefix)/share/zsh-highlighter/zsh-highlighter.zsh"
    EOS
  end
end

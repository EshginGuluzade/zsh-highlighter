class ZshHighlighter < Formula
  desc "Minimal, ultra-fast zsh syntax highlighter written in Rust"
  homepage "https://github.com/EshginGuluzade/zsh-highlighter"
  version "0.0.1"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/EshginGuluzade/zsh-highlighter/releases/download/v0.0.1/zsh-highlighter-v0.0.1-aarch64-apple-darwin.tar.gz"
    sha256 "4c49cb30992594c9f5866bc568ede0044228129159ba771056a48bd031db6120"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/EshginGuluzade/zsh-highlighter/releases/download/v0.0.1/zsh-highlighter-v0.0.1-x86_64-apple-darwin.tar.gz"
    sha256 "3b87adda40b626a30cb57be9ce58a7680bf7a3eac4ea1dba98ef5fc5901e2539"
  elsif OS.linux?
    url "https://github.com/EshginGuluzade/zsh-highlighter/releases/download/v0.0.1/zsh-highlighter-v0.0.1-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "b2f530b8ad1c9f105b8152e40141216b56e1322c7d866a5c59b6a467015878db"
  end

  def install
    bin.install "zsh-highlighter"
    (share/"zsh-highlighter").install "zsh-highlighter.zsh"
  end

  def caveats
    <<~EOS
      To activate zsh-highlighter, add the following to your ~/.zshrc:

        source "$(brew --prefix)/share/zsh-highlighter/zsh-highlighter.zsh"
    EOS
  end
end

cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.34"
  sha256 arm:   "802b7ee9b81cc1390b858fced78a6d6f725c17b851dda9349376261e7d40cfe5",
         intel: "b0ca13dca259dbdf047682bcfdf098373caee4b4abb0f1704686ab5f3dc9e396"

  url "https://github.com/lassejlv/termy/releases/download/v#{version}/Termy-v#{version}-macos-#{arch}.dmg"
  name "Termy"
  desc "Minimal GPU-powered terminal written in Rust"
  homepage "https://github.com/lassejlv/termy"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  app "Termy.app"
end

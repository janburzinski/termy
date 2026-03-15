cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.53"
  sha256 arm:   "d9e297d69d6ddbf8747c1e03e8cabd53a011bcc191cc3fa7d61acfad2345fc77",
         intel: "962fd2e42d820b6287f550f2688f3ff0aa8a50afae27c38f24d7b8d0d68b9add"

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

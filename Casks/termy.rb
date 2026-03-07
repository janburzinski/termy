cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.45"
  sha256 arm:   "c897a767b66568e8db7257f22f2c849da441b11c959b2524780badf6d143755c",
         intel: "d35c6d309cb18764273f50ce81b762b19a06ab14069c3681ad891e00eed48781"

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

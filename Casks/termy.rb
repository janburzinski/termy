cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.62"
  sha256 arm:   "c223f848eb3884f7b9db1bbdfef8d080db8f13b905113f32611552d36f372018",
         intel: "dbaee7997c8b66e4c9ec8e8a47c22037b405f676efd62b30aa86b0e0c1ef045f"

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

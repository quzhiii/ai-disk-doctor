class Aidisk < Formula
  desc "AI-era disk space diagnostics and governance CLI"
  homepage "https://github.com/quzhiii/ai-disk-doctor"
  version "1.6.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_intel do
      url "https://github.com/quzhiii/ai-disk-doctor/releases/download/v#{version}/aidisk-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "TO_BE_FILLED_FROM_RELEASE_ARTIFACT"
    end

    on_arm do
      url "https://github.com/quzhiii/ai-disk-doctor/releases/download/v#{version}/aidisk-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "TO_BE_FILLED_FROM_RELEASE_ARTIFACT"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/quzhiii/ai-disk-doctor/releases/download/v#{version}/aidisk-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "TO_BE_FILLED_FROM_RELEASE_ARTIFACT"
    end

    on_arm do
      url "https://github.com/quzhiii/ai-disk-doctor/releases/download/v#{version}/aidisk-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "TO_BE_FILLED_FROM_RELEASE_ARTIFACT"
    end
  end

  def install
    bin.install "aidisk"
  end

  test do
    system "#{bin}/aidisk", "--help"
    system "#{bin}/aidisk", "scan", "--help"
  end
end

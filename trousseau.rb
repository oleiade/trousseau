require 'formula'

class Trousseau < Formula
  homepage 'https://github.com/oleiade/trousseau'
  url 'https://github.com/oleiade/trousseau/releases/download/0.4.0/trousseau_0.4.0_darwin_amd64.zip'
  sha256 '9f4a794ad5427ef56c03dd1052c2f1cd236fe4655a7e88a841f67183ea12f00f'

  def install
    bin.install('trousseau')
  end
end

require 'formula'

class Trousseau < Formula
  homepage 'https://github.com/oleiade/trousseau'
  url 'https://github.com/oleiade/trousseau/archive/0.1.4.tar.gz'
  sha1 '550e38eccaf4ab2c35afc1ed3f997d9ff2f15b89'

  depends_on 'go' => :build
  depends_on 'mercurial' => :build
  depends_on 'bzr' => :build

  def install
    system 'make', 'all'
    bin.install('bin/trousseau')
  end
end

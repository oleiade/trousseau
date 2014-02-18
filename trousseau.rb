require 'formula'

class Trousseau < Formula
  homepage 'https://github.com/oleiade/trousseau'
  url 'https://github.com/oleiade/trousseau/archive/0.2.3.tar.gz'
  sha1 '66d0b79525c5bed3e1d6b791c90ee77b0a43c487'

  depends_on 'go' => :build
  depends_on 'mercurial' => :build
  depends_on 'bzr' => :build

  def install
    system 'make', 'all'
    bin.install('bin/trousseau')
  end
end

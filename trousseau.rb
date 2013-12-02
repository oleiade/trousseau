require 'formula'

class Trousseau < Formula
  homepage 'https://github.com/oleiade/trousseau'
  url 'https://github.com/oleiade/trousseau/archive/0.2.0.tar.gz'
  sha1 ''

  depends_on 'go' => :build
  depends_on 'mercurial' => :build
  depends_on 'bzr' => :build

  def install
    system 'make', 'all'
    bin.install('bin/trousseau')
  end
end

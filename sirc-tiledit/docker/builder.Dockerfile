FROM docker.io/library/ubuntu:noble

LABEL version="1.0"
LABEL maintainer="Sean Dawson <contact@seandawson.info>"
LABEL description="This image is used as a consistent environment to build the C++/qt6 based tiledit component of the SIRC project."

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y \
      # Used by Meson to find dependencies
      pkg-config \
      # Application dependencies
      libpng-dev qt6-base-dev  \
      # Needed to install llvm
      wget software-properties-common \
      # pip is needed to get up-to-date version of python dependencies (the ones fetched via apt can be out of date)
      python3-pip \
      # git is needed so that meson can use 'git ls-files' to work out what files are checked in to git so it doesn't try to lint/format check unrelated files
      git && \
    wget https://apt.llvm.org/llvm.sh -P /tmp && \
    chmod +x /tmp/llvm.sh && \
    /tmp/llvm.sh 18 all && \
    # TODO: Surely there is an easier way to set default version
    ln -s $(which clang-tidy-18) /usr/local/bin/clang-tidy && \
    ln -s $(which clang-format-18) /usr/local/bin/clang-format && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN python3 -m pip install --break-system-packages meson ninja gcovr

RUN update-alternatives --install \
        /usr/bin/llvm-config       llvm-config      /usr/bin/llvm-config-18  200 \
--slave /usr/bin/llvm-ar           llvm-ar          /usr/bin/llvm-ar-18 \
--slave /usr/bin/llvm-as           llvm-as          /usr/bin/llvm-as-18 \
--slave /usr/bin/llvm-bcanalyzer   llvm-bcanalyzer  /usr/bin/llvm-bcanalyzer-18 \
--slave /usr/bin/llvm-cov          llvm-cov         /usr/bin/llvm-cov-18 \
--slave /usr/bin/llvm-diff         llvm-diff        /usr/bin/llvm-diff-18 \
--slave /usr/bin/llvm-dis          llvm-dis         /usr/bin/llvm-dis-18 \
--slave /usr/bin/llvm-dwarfdump    llvm-dwarfdump   /usr/bin/llvm-dwarfdump-18 \
--slave /usr/bin/llvm-extract      llvm-extract     /usr/bin/llvm-extract-18 \
--slave /usr/bin/llvm-link         llvm-link        /usr/bin/llvm-link-18 \
--slave /usr/bin/llvm-mc           llvm-mc          /usr/bin/llvm-mc-18 \
--slave /usr/bin/llvm-mcmarkup     llvm-mcmarkup    /usr/bin/llvm-mcmarkup-18 \
--slave /usr/bin/llvm-nm           llvm-nm          /usr/bin/llvm-nm-18 \
--slave /usr/bin/llvm-objdump      llvm-objdump     /usr/bin/llvm-objdump-18 \
--slave /usr/bin/llvm-ranlib       llvm-ranlib      /usr/bin/llvm-ranlib-18 \
--slave /usr/bin/llvm-readobj      llvm-readobj     /usr/bin/llvm-readobj-18 \
--slave /usr/bin/llvm-rtdyld       llvm-rtdyld      /usr/bin/llvm-rtdyld-18 \
--slave /usr/bin/llvm-size         llvm-size        /usr/bin/llvm-size-18 \
--slave /usr/bin/llvm-stress       llvm-stress      /usr/bin/llvm-stress-18 \
--slave /usr/bin/llvm-symbolizer   llvm-symbolizer  /usr/bin/llvm-symbolizer-18 \
--slave /usr/bin/llvm-tblgen       llvm-tblgen      /usr/bin/llvm-tblgen-18

RUN groupadd -g 10001 builder && \
   useradd -u 10000 -g builder builder && \
   mkdir /builder && \
   chown -R builder:builder /builder

COPY ./entrypoint.sh /builder/

WORKDIR /builder

USER builder:builder

ENTRYPOINT ["/builder/entrypoint.sh"]
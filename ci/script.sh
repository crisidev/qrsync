# This script takes care of testing your crate

set -ex

main() {
    # build the target relase
    cross build --target "${TARGET}" --release

    if [ ! -z "${DISABLE_TESTS}" ]; then
        return
    fi

    # run the tests
    cross test --target "${TARGET}" --release

    # run the executable to ensure is has been built properly
    cross run --target "${TARGET}" --release -- --version
}

# we don't run the "test phase" when doing deploys
if [ -z "${TRAVIS_TAG}" ]; then
    main
fi

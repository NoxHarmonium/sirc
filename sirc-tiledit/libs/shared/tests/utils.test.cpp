
#include <iostream>
#include <vector>

#include <utils.hpp>

#include "catch2/catch_amalgamated.hpp"

TEST_CASE("safeCastIntVector: Converts vector correctly") {
  const std::vector<size_t> input = {1, 2, 3, 4, 5};

  const auto output = safeCastIntVector<uint16_t>(std::span(input));

  REQUIRE(output == std::vector<uint16_t>({1, 2, 3, 4, 5}));
}

TEST_CASE("safeCastIntVector:Converts array correctly") {
  const std::array<size_t, 5> input = {1, 2, 3, 4, 5};

  const auto output =
      safeCastIntVector<uint16_t, const size_t>(std::span(input));

  REQUIRE(output == std::vector<uint16_t>({1, 2, 3, 4, 5}));
}

TEST_CASE("safeCastIntVector: Throws when values are out of range") {
  const std::vector<size_t> input = {1, 2, 0xFFFFFFFF, 4, 5};

  const auto doTest = [&input]() {
    const auto output = safeCastIntVector<uint16_t>(std::span(input));
  };

  REQUIRE_THROWS_AS(doTest(), std::invalid_argument);
}

TEST_CASE("packIntVector: Converts vector correctly") {
  const std::vector<size_t> input = {1, 2, 3, 4, 5, 6, 7, 8};

  const auto output = packIntVector<uint16_t>(std::span(input), 4);

  REQUIRE(output == std::vector<uint16_t>({0x1234, 0x5678}));
}

TEST_CASE("packIntVector: Throws number of items to pack do not fit evenly "
          "into output type") {
  const std::vector<size_t> input = {1, 2, 3, 4, 5};

  const auto doTest = [&input]() {
    const auto output = packIntVector<uint16_t>(std::span(input), 4);
  };

  REQUIRE_THROWS_AS(doTest(), std::invalid_argument);
}

// TODO: More tests for edge cases
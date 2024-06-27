#include <sircimage.hpp>

SircImage::SircImage(SircImageData imageData)
    : imageData(std::move(imageData)) {}

// TODO: This breaks encapsulation I suppose, possibly making this class kind of
// pointless. Might need to revisit
SircImageData SircImage::getImageData() const { return this->imageData; }

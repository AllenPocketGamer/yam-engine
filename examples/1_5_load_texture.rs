// use yam::assets::AssetsLoader;
use image::GenericImageView;

fn main() {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("assets/images/png/test.png").unwrap();

    // The dimensions method returns the images width and height.
    println!("dimensions: {:?}", img.dimensions());

    // The color method returns the image's `ColorType`.
    println!("{:?}", img.color());

    // Write the contents of this image to the Write in JPG format.
    img.save("to_jpg.jpg").unwrap();
}
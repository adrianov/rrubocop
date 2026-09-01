@stories = Story.includes(:version_filter)
                .includes(slides: { big_image_attachment: :blob })
                .with_attached_small_image
                .ordered

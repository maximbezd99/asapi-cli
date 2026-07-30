CREATE INDEX reviews_by_rating
    ON reviews(app_id, country, rating, updated_at DESC, review_id DESC);

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use symphonia::core::meta::{Value, Visual, Tag, StandardTagKey};

use crate::lang::db::*;

const BASE64_CONF: base64::Config = base64::Config::new(base64::CharacterSet::Standard, false);

pub struct Tags {
    data: HashMap<String, TagType>,
    filename: PathBuf,
}

#[inline]
fn std_tag_to_str(key: StandardTagKey) -> &'static str {
    match key {
        StandardTagKey::AcoustidFingerprint => "acoustid_fingerprint",
        StandardTagKey::AcoustidId => "acoustid_id",
        StandardTagKey::Album => "album",
        StandardTagKey::AlbumArtist => "albumartist",
        StandardTagKey::Arranger => "arranger",
        StandardTagKey::Artist => "artist",
        StandardTagKey::Bpm => "bpm",
        StandardTagKey::Comment => "comment",
        StandardTagKey::Compilation => "compilation",
        StandardTagKey::Composer => "composer",
        StandardTagKey::Conductor => "conductor",
        StandardTagKey::ContentGroup => "contentgroup",
        StandardTagKey::Copyright => "copyright",
        StandardTagKey::Date => "date",
        StandardTagKey::Description => "description",
        StandardTagKey::DiscNumber => "disc",
        StandardTagKey::DiscSubtitle => "disc_subtitle",
        StandardTagKey::DiscTotal => "disc_total",
        StandardTagKey::EncodedBy => "encoded_by",
        StandardTagKey::Encoder => "encoder",
        StandardTagKey::EncoderSettings => "encoder_settings",
        StandardTagKey::EncodingDate => "encoding_date",
        StandardTagKey::Engineer => "engineer",
        StandardTagKey::Ensemble => "ensemble",
        StandardTagKey::Genre => "genre",
        StandardTagKey::IdentAsin => "ident_asin",
        StandardTagKey::IdentBarcode => "ident_barcode",
        StandardTagKey::IdentCatalogNumber => "ident_catalog_number",
        StandardTagKey::IdentEanUpn => "ident_ean_upn",
        StandardTagKey::IdentIsrc => "ident_isrc",
        StandardTagKey::IdentPn => "ident_pn",
        StandardTagKey::IdentPodcast => "ident_podcast",
        StandardTagKey::IdentUpc => "ident_upc",
        StandardTagKey::Label => "label",
        StandardTagKey::Language => "language",
        StandardTagKey::License => "license",
        StandardTagKey::Lyricist => "lyricist",
        StandardTagKey::Lyrics => "lyrics",
        StandardTagKey::MediaFormat => "mediaformat",
        StandardTagKey::MixDj => "mixdj",
        StandardTagKey::MixEngineer => "mix_engineer",
        StandardTagKey::Mood => "mood",
        StandardTagKey::MovementName => "movement_name",
        StandardTagKey::MovementNumber => "movement_number",
        StandardTagKey::MusicBrainzAlbumArtistId => "MusicBrainz_albumartist_id",
        StandardTagKey::MusicBrainzAlbumId => "MusicBrainz_album_id",
        StandardTagKey::MusicBrainzArtistId => "MusicBrainz_artist_id",
        StandardTagKey::MusicBrainzDiscId => "MusicBrainz_disc_id",
        StandardTagKey::MusicBrainzGenreId => "MusicBrainz_genre_id",
        StandardTagKey::MusicBrainzLabelId => "MusicBrainz_label_id",
        StandardTagKey::MusicBrainzOriginalAlbumId => "MusicBrainz_original_album_id",
        StandardTagKey::MusicBrainzOriginalArtistId => "MusicBrainz_original_artist_id",
        StandardTagKey::MusicBrainzRecordingId => "MusicBrainz_recording_id",
        StandardTagKey::MusicBrainzReleaseGroupId => "MusicBrainz_release_group_id",
        StandardTagKey::MusicBrainzReleaseStatus => "MusicBrainz_release_status",
        StandardTagKey::MusicBrainzReleaseTrackId => "MusicBrainz_release_track_id",
        StandardTagKey::MusicBrainzReleaseType => "MusicBrainz_release_type",
        StandardTagKey::MusicBrainzTrackId => "MusicBrainz_track_id",
        StandardTagKey::MusicBrainzWorkId => "MusicBrainz_work_id",
        StandardTagKey::Opus => "Opus",
        StandardTagKey::OriginalAlbum => "original_album",
        StandardTagKey::OriginalArtist => "original_artist",
        StandardTagKey::OriginalDate => "original_date",
        StandardTagKey::OriginalFile => "original_file",
        StandardTagKey::OriginalWriter => "original_writer",
        StandardTagKey::Owner => "owner",
        StandardTagKey::Part => "part",
        StandardTagKey::PartTotal => "part_total",
        StandardTagKey::Performer => "performer",
        StandardTagKey::Podcast => "podcast",
        StandardTagKey::PodcastCategory => "podcast_category",
        StandardTagKey::PodcastDescription => "podcast_description",
        StandardTagKey::PodcastKeywords => "podcast_keywords",
        StandardTagKey::Producer => "producer",
        StandardTagKey::PurchaseDate => "purchase_date",
        StandardTagKey::Rating => "rating",
        StandardTagKey::ReleaseCountry => "release_country",
        StandardTagKey::ReleaseDate => "release_date",
        StandardTagKey::Remixer => "remixer",
        StandardTagKey::ReplayGainAlbumGain => "ReplayGain_album_gain",
        StandardTagKey::ReplayGainAlbumPeak => "ReplayGain_album_peak",
        StandardTagKey::ReplayGainTrackGain => "ReplayGain_track_gain",
        StandardTagKey::ReplayGainTrackPeak => "ReplayGain_track_peak",
        StandardTagKey::Script => "script",
        StandardTagKey::SortAlbum => "sort_album",
        StandardTagKey::SortAlbumArtist => "sort_albumartist",
        StandardTagKey::SortArtist => "sort_artist",
        StandardTagKey::SortComposer => "sort_composer",
        StandardTagKey::SortTrackTitle => "sort_title",
        StandardTagKey::TaggingDate => "tagging_date",
        StandardTagKey::TrackNumber => "track",
        StandardTagKey::TrackSubtitle => "track_subtitle",
        StandardTagKey::TrackTitle => "title",
        StandardTagKey::TrackTotal => "track_total",
        StandardTagKey::TvEpisode => "tv_episode",
        StandardTagKey::TvEpisodeTitle => "tv_episode_title",
        StandardTagKey::TvNetwork => "tv_network",
        StandardTagKey::TvSeason => "tv_season",
        StandardTagKey::TvShowTitle => "tv_show_title",
        StandardTagKey::Url => "url",
        StandardTagKey::UrlArtist => "url_artist",
        StandardTagKey::UrlCopyright => "url_copyright",
        StandardTagKey::UrlInternetRadio => "url_internet_radio",
        StandardTagKey::UrlLabel => "url_label",
        StandardTagKey::UrlOfficial => "url_official",
        StandardTagKey::UrlPayment => "url_payment",
        StandardTagKey::UrlPodcast => "url_podcast",
        StandardTagKey::UrlPurchase => "url_purchase",
        StandardTagKey::UrlSource => "url_source",
        StandardTagKey::Version => "version",
        StandardTagKey::Writer => "writer",
    }
}

impl Tags {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            data: HashMap::new(),
            filename: path.as_ref().canonicalize().unwrap(),
        }
    }

    pub fn add(&mut self, tag: &Tag) {
        let key = if let Some(std_key) = tag.std_key {
            std_tag_to_str(std_key).to_owned()
        } else {
            tag.key.clone()
        };
        let value = &tag.value;
        if let Some(tag_type) = TagType::from_symphonia_value(value) {
            self.data.insert(key.trim().to_lowercase(), tag_type);
        }
    }

    pub fn add_visual(&mut self, visual: &Visual) {
        if let Some(tag_type) = TagType::from_symphonia_visual(visual) {
            self.data.insert("cover".to_owned(), tag_type);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn track_title(&self) -> String {
        self.data
            .get(std_tag_to_str(StandardTagKey::TrackTitle))
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_title())
    }

    #[inline]
    fn default_title(&self) -> String {
        let extension = self
            .filename
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        self.filename
            .file_name()
            .and_then(|file| file.to_str())
            .map(|file| file.replacen(&format!(".{}", extension), "", 1))
            .unwrap_or_else(|| "Unknown Title".into())
    }

    #[inline]
    pub fn artist_name(&self) -> Option<String> {
        self.data
            .get(std_tag_to_str(StandardTagKey::Artist))
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn album_title(&self) -> Option<String> {
        self.data
            .get(std_tag_to_str(StandardTagKey::Album))
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn albumartist_name(&self) -> Option<String> {
        self.data
            .get(std_tag_to_str(StandardTagKey::AlbumArtist))
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn genre_title(&self) -> Option<String> {
        self.data
            .get(std_tag_to_str(StandardTagKey::Genre))
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn track_number(&self) -> Option<u64> {
        self.data
            .get(std_tag_to_str(StandardTagKey::TrackNumber))
            .unwrap_or(&TagType::Unknown)
            .uint()
    }

    #[inline]
    pub fn cover_art(&self) -> Option<String> {
        self.data
            .get("cover")
            .unwrap_or(&TagType::Unknown)
            .str()
            .map(|s| s.to_string())
    }

    #[inline]
    pub fn track_date(&self) -> Option<u64> {
        self.data
            .get(std_tag_to_str(StandardTagKey::Date))
            .unwrap_or(&TagType::Unknown)
            .uint()
    }

    pub fn song(
        &self,
        id: u64,
        artist_id: u64,
        album_id: Option<u64>,
        meta_id: u64,
        genre_id: u64,
    ) -> DbMusicItem {
        DbMusicItem {
            song_id: id,
            title: self.track_title(),
            artist: artist_id,
            album: album_id,
            filename: format!("file://{}", self.filename.to_str().unwrap_or("")),
            metadata: meta_id,
            genre: genre_id,
        }
    }

    pub fn meta(&self, id: u64) -> DbMetaItem {
        DbMetaItem {
            meta_id: id,
            plays: self
                .data
                .get("plays")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            track: self.track_number().unwrap_or(id),
            disc: self
                .data
                .get(std_tag_to_str(StandardTagKey::DiscNumber))
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(1),
            duration: self
                .data
                .get("duration")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            date: self.track_date().unwrap_or(0),
        }
    }

    pub fn artist(&self, id: u64, genre_id: u64) -> DbArtistItem {
        DbArtistItem {
            artist_id: id,
            name: self
                .artist_name()
                .unwrap_or_else(|| "Unknown Artist".into()),
            genre: genre_id,
        }
    }

    pub fn album_artist(&self, id: u64, genre_id: u64) -> DbArtistItem {
        DbArtistItem {
            artist_id: id,
            name: self.albumartist_name()
                .unwrap_or("Unknown Artist".into()),
            genre: genre_id,
        }
    }

    pub fn album(&self, id: u64, meta_id: u64, artist_id: u64, genre_id: u64) -> DbAlbumItem {
        DbAlbumItem {
            album_id: id,
            title: self.album_title().unwrap_or_else(|| "Unknown Album".into()),
            metadata: meta_id,
            artist: artist_id,
            genre: genre_id,
        }
    }

    pub fn album_meta(&self, id: u64) -> DbMetaItem {
        DbMetaItem {
            meta_id: id,
            plays: self
                .data
                .get("plays")
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            track: self
                .data
                .get(std_tag_to_str(StandardTagKey::TrackTotal))
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
            disc: self
                .data
                .get(std_tag_to_str(StandardTagKey::DiscTotal))
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(1),
            duration: 0,
            date: self
                .data
                .get(std_tag_to_str(StandardTagKey::Date))
                .unwrap_or(&TagType::Unknown)
                .uint()
                .unwrap_or(0),
        }
    }

    pub fn genre(&self, id: u64) -> DbGenreItem {
        DbGenreItem {
            genre_id: id,
            title: self.genre_title().unwrap_or_else(|| "Unknown Genre".into()),
        }
    }

    pub fn export_to_item(self, item: &mut crate::Item, overwrite: bool) {
        for (key, val) in self.data {
            if let Some(primitive_val) = val.to_primitive() {
                if overwrite || item.field(&key).is_none()  {
                    item.set_field(&key, primitive_val);
                }
            }
        }
        if overwrite || item.field("filename").is_none()  {
            item.set_field("filename", self.filename.display().to_string().into());
        }
    }
}

#[derive(Clone)]
enum TagType {
    Boolean(bool),
    Flag,
    I64(i64),
    U64(u64),
    Str(String),
    Unknown,
}

impl TagType {
    #[inline]
    fn from_symphonia_value(value: &Value) -> Option<Self> {
        match value {
            Value::Binary(_val) => None,
            Value::Boolean(b) => Some(Self::Boolean(*b)),
            Value::Flag => Some(Self::Flag),
            Value::Float(_val) => None,
            Value::SignedInt(i) => Some(Self::I64(*i)),
            Value::String(s) => Some(Self::Str(s.clone())),
            Value::UnsignedInt(u) => Some(Self::U64(*u)),
        }
    }

    #[inline]
    fn from_symphonia_visual(visual: &Visual) -> Option<Self> {
        Some(Self::Str(format!("data:{};base64,{}", &visual.media_type, base64::encode_config(&visual.data, BASE64_CONF))))
    }

    fn str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    fn uint(&self) -> Option<u64> {
        match self {
            Self::I64(i) => (*i).try_into().ok(),
            Self::U64(u) => Some(*u),
            Self::Str(s) => s.parse::<u64>().ok(),
            _ => None,
        }
    }

    fn to_primitive(self) -> Option<crate::lang::TypePrimitive> {
        match self {
            Self::Boolean(b) => Some(crate::lang::TypePrimitive::Bool(b)),
            Self::Flag => None,
            Self::I64(i) => Some(crate::lang::TypePrimitive::Int(i)),
            Self::U64(u) => Some(crate::lang::TypePrimitive::UInt(u)),
            Self::Str(s) => Some(crate::lang::TypePrimitive::String(s.clone())),
            Self::Unknown => None,
        }
    }
}

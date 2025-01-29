use jni::{errors::Result, objects::JObject, JNIEnv};

/// A messaging object you can use to request an action from another android app component.
pub struct Intent<'env> {
    env: JNIEnv<'env>,
    object: JObject<'env>,
}

impl<'env> Intent<'env> {
    pub fn from_object(env: JNIEnv<'env>, object: JObject<'env>) -> Self {
        Self { env, object }
    }

    pub fn new(env: JNIEnv<'env>, action: impl AsRef<str>) -> Result<Self> {
        let intent_class = env.find_class("android/content/Intent")?;
        let action_view =
            env.get_static_field(intent_class, action.as_ref(), "Ljava/lang/String;")?;

        let intent = env.new_object(intent_class, "(Ljava/lang/String;)V", &[action_view])?;

        Ok(Self {
            env,
            object: intent,
        })
    }

    pub fn new_with_uri(
        env: JNIEnv<'env>,
        action: impl AsRef<str>,
        uri: impl AsRef<str>,
    ) -> Result<Self> {
        let url_string = env.new_string(uri)?;
        let uri_class = env.find_class("android/net/Uri")?;
        let uri = env.call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[url_string.into()],
        )?;

        let intent_class = env.find_class("android/content/Intent")?;
        let action_view =
            env.get_static_field(intent_class, action.as_ref(), "Ljava/lang/String;")?;

        let intent = env.new_object(
            intent_class,
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[action_view, uri],
        )?;

        Ok(Self {
            env,
            object: intent,
        })
    }

    /// Set the class name for the intent target.
    /// ```no_run
    /// use android_intent::{Action, Extra, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send).unwrap()
    ///     .with_class_name("com.excample", "IntentTarget").unwrap();
    /// # })
    /// ```
    pub fn with_class_name(
        self,
        package_name: impl AsRef<str>,
        class_name: impl AsRef<str>,
    ) -> Result<Self> {
        let package_name = self.env.new_string(package_name)?;
        let class_name = self.env.new_string(class_name)?;

        self.env.call_method(
            self.object,
            "setClassName",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[package_name.into(), class_name.into()],
        )?;

        Ok(self)
    }

    /// Add extended data to the intent.
    /// ```no_run
    /// use android_intent::{Action, Extra, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send).unwrap()
    ///     .with_extra(Extra::Text, "Hello World!").unwrap();
    /// # })
    /// ```
    pub fn with_extra(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        let key = self.env.new_string(key)?;
        let value = self.env.new_string(value)?;

        self.env.call_method(
            self.object,
            "putExtra",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[key.into(), value.into()],
        )?;

        Ok(self)
    }

    /// Builds a new [`super::Action::Chooser`] Intent that wraps the given target intent.
    /// ```no_run
    /// use android_intent::{Action, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send).unwrap()
    ///     .into_chooser().unwrap();
    /// # })
    /// ```
    // TODO: Rename to with_?
    pub fn into_chooser(self) -> Result<Self> {
        self.into_chooser_with_title(None::<&str>)
    }

    // TODO: Rename to with_?
    pub fn into_chooser_with_title(mut self, title: Option<impl AsRef<str>>) -> Result<Self> {
        let title_value = if let Some(title) = title {
            let s = self.env.new_string(title)?;
            s.into()
        } else {
            JObject::null().into()
        };

        let intent_class = self.env.find_class("android/content/Intent")?;
        let intent = self.env.call_static_method(
            intent_class,
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[self.object.into(), title_value],
        )?;

        self.object = intent.try_into()?;
        Ok(self)
    }

    /// Set an explicit MIME data type.
    /// ```no_run
    /// use android_intent::{Action, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send).unwrap()
    ///     .with_type("text/plain").unwrap();
    /// # })
    /// ```
    pub fn with_type(self, type_name: impl AsRef<str>) -> Result<Self> {
        let jstring = self.env.new_string(type_name)?;

        self.env.call_method(
            self.object,
            "setType",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[jstring.into()],
        )?;

        Ok(self)
    }

    pub fn start_activity(self) -> Result<()> {
        let cx = ndk_context::android_context();
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        self.env.call_method(
            activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[self.object.into()],
        )?;

        Ok(())
    }
}

/// Builder for intents that allows to capture [`Result`] at the end.
#[must_use]
pub struct IntentBuilder<'env> {
    inner: Result<Intent<'env>>,
}

impl<'env> IntentBuilder<'env> {
    pub fn from_object(env: JNIEnv<'env>, object: JObject<'env>) -> Self {
        Self {
            inner: Ok(Intent::from_object(env, object)),
        }
    }

    fn from_fn(f: impl FnOnce() -> Result<Intent<'env>>) -> Self {
        let inner = f();
        Self { inner }
    }

    pub fn new(env: JNIEnv<'env>, action: impl AsRef<str>) -> Self {
        Self::from_fn(|| Intent::new(env, action))
    }

    pub fn new_with_uri(env: JNIEnv<'env>, action: impl AsRef<str>, uri: impl AsRef<str>) -> Self {
        Self::from_fn(|| Intent::new_with_uri(env, action, uri))
    }

    /// Set the class name for the intent target.
    /// ```no_run
    /// use android_intent::{Action, Extra, IntentBuilder};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = IntentBuilder::new(env, Action::Send)
    ///     .with_class_name("com.example", "IntentTarget");
    /// # })
    /// ```
    pub fn with_class_name(
        self,
        package_name: impl AsRef<str>,
        class_name: impl AsRef<str>,
    ) -> Self {
        self.and_then(|inner| inner.with_class_name(package_name, class_name))
    }

    /// Add extended data to the intent.
    /// ```no_run
    /// use android_intent::{Action, Extra, IntentBuilder};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = IntentBuilder::new(env, Action::Send)
    ///     .with_extra(Extra::Text, "Hello World!");
    /// # })
    /// ```
    pub fn with_extra(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.and_then(|inner| inner.with_extra(key, value))
    }

    /// Builds a new [`super::Action::Chooser`] Intent that wraps the given target intent.
    /// ```no_run
    /// use android_intent::{Action, IntentBuilder};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = IntentBuilder::new(env, Action::Send).into_chooser();
    /// # })
    /// ```
    // TODO: Rename to with_?
    pub fn into_chooser(self) -> Self {
        self.into_chooser_with_title(None::<&str>)
    }

    // TODO: Rename to with_?
    pub fn into_chooser_with_title(self, title: Option<impl AsRef<str>>) -> Self {
        self.and_then(|inner| inner.into_chooser_with_title(title))
    }

    /// Set an explicit MIME data type.
    /// ```no_run
    /// use android_intent::{Action, IntentBuilder};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = IntentBuilder::new(env, Action::Send)
    ///     .with_type("text/plain");
    /// # })
    /// ```
    pub fn with_type(self, type_name: impl AsRef<str>) -> Self {
        self.and_then(|inner| inner.with_type(type_name))
    }

    pub fn start_activity(self) -> Result<()> {
        self.inner.and_then(|inner| inner.start_activity())
    }

    fn and_then(mut self, f: impl FnOnce(Intent) -> Result<Intent>) -> Self {
        self.inner = self.inner.and_then(f);
        self
    }
}

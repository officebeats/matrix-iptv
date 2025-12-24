
    pub fn next_series_episode(&mut self) {
        if self.series_episodes.is_empty() {
            return;
        }
        let i = match self.series_episode_list_state.selected() {
            Some(i) => {
                if i >= self.series_episodes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_series_episode_index = i;
        self.series_episode_list_state.select(Some(i));
    }

    pub fn previous_series_episode(&mut self) {
        if self.series_episodes.is_empty() {
            return;
        }
        let i = match self.series_episode_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.series_episodes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_series_episode_index = i;
        self.series_episode_list_state.select(Some(i));
    }

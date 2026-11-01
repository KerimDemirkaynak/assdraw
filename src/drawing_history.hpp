// A deliberately small history container for document/drawing state only.
// New code in this file is licensed under the BSD 3-Clause License; see LICENSE.
#pragma once

#include <vector>

template <typename Snapshot>
class DrawingHistory
{
public:
    void Push(const Snapshot& snapshot)
    {
        undo_.push_back(snapshot);
        redo_.clear();
    }

    bool Undo(const Snapshot& current, Snapshot& restored)
    {
        if (undo_.empty())
            return false;
        restored = undo_.back();
        undo_.pop_back();
        redo_.push_back(current);
        return true;
    }

    bool Redo(const Snapshot& current, Snapshot& restored)
    {
        if (redo_.empty())
            return false;
        restored = redo_.back();
        redo_.pop_back();
        undo_.push_back(current);
        return true;
    }

    bool CanUndo() const { return !undo_.empty(); }
    bool CanRedo() const { return !redo_.empty(); }
    const Snapshot& TopUndo() const { return undo_.back(); }
    const Snapshot& TopRedo() const { return redo_.back(); }

private:
    std::vector<Snapshot> undo_;
    std::vector<Snapshot> redo_;
};

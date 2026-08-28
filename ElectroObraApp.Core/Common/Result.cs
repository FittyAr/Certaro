namespace ElectroObraApp.Core.Common;

public class Result
{
    public bool IsSuccess { get; }
    public string? Error { get; }
    public IReadOnlyList<string> Errors { get; }

    protected Result(bool isSuccess, string? error, IReadOnlyList<string>? errors = null)
    {
        IsSuccess = isSuccess;
        Error = error;
        Errors = errors ?? (error is null ? Array.Empty<string>() : new[] { error });
    }

    public static Result Success() => new(true, null);

    public static Result Failure(string error) => new(false, error);

    public static Result Failure(IEnumerable<string> errors)
    {
        var list = errors.ToList();
        return new Result(false, list.FirstOrDefault(), list);
    }
}

public class Result<T> : Result
{
    public T? Value { get; }

    private Result(T value) : base(true, null)
    {
        Value = value;
    }

    private Result(string error) : base(false, error)
    {
    }

    private Result(IEnumerable<string> errors) : base(false, errors.FirstOrDefault(), errors.ToList())
    {
    }

    public static Result<T> Success(T value) => new(value);

    public new static Result<T> Failure(string error) => new(error);

    public new static Result<T> Failure(IEnumerable<string> errors) => new(errors);
}
